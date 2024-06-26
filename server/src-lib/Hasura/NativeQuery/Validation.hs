-- | Generic validation of native queries while tracking them.
module Hasura.NativeQuery.Validation
  ( validateArgumentDeclaration,
    DisableNativeQueryValidation (..),
  )
where

import Data.Aeson qualified as J
import Data.HashMap.Strict qualified as HashMap
import Data.Set (Set)
import Data.Set qualified as Set
import Hasura.Base.Error
import Hasura.Base.ErrorMessage (fromErrorMessage)
import Hasura.Base.ToErrorValue (toErrorValue)
import Hasura.NativeQuery.InterpolatedQuery
import Hasura.NativeQuery.Metadata
import Hasura.Prelude hiding (first)

-- | Check that the set of declared arguments and the set of used arguments (in the code)
--   is the same.
validateArgumentDeclaration ::
  (MonadIO m, MonadError QErr m) =>
  NativeQueryMetadata b ->
  m ()
validateArgumentDeclaration NativeQueryMetadata {_nqmCode, _nqmArguments} = do
  let occurringArguments :: Set ArgumentName
      occurringArguments = getUniqueVariables _nqmCode

      declaredArguments :: Set ArgumentName
      declaredArguments = Set.fromList $ HashMap.keys _nqmArguments

      undeclaredArguments :: Set ArgumentName
      undeclaredArguments = occurringArguments `Set.difference` declaredArguments

      unusedArguments :: Set ArgumentName
      unusedArguments = declaredArguments `Set.difference` occurringArguments
  unless (Set.null undeclaredArguments)
    $ throwError
    $ err400 ValidationFailed
    $ fromErrorMessage
    $ "The following columns are used in the query, but are not declared as arguments: "
    <> toErrorValue undeclaredArguments

  unless (Set.null unusedArguments)
    $ throwError
    $ err400 ValidationFailed
    $ fromErrorMessage
    $ "The following columns are declared as arguments, but are not used in the query: "
    <> toErrorValue unusedArguments

-- | Should we validate Native Queries against the database?
-- Avoiding that could speed-up schema cache rebuild significantly.
data DisableNativeQueryValidation
  = AlwaysValidateNativeQueries
  | NeverValidateNativeQueries
  deriving (Eq, Show)

instance J.FromJSON DisableNativeQueryValidation where
  parseJSON =
    J.withBool "DisableNativeQueryValidation"
      $ pure
      . \case
        True -> NeverValidateNativeQueries
        False -> AlwaysValidateNativeQueries

instance J.ToJSON DisableNativeQueryValidation where
  toJSON = \case
    AlwaysValidateNativeQueries -> J.Bool False
    NeverValidateNativeQueries -> J.Bool True
