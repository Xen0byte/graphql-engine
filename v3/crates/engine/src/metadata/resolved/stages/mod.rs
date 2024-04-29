pub mod boolean_expressions;
pub mod commands;
/// This is where we'll be moving explicit metadata resolve stages
pub mod data_connector_scalar_types;
pub mod data_connector_type_mappings;
pub mod data_connectors;
pub mod graphql_config;
pub mod models;
pub mod scalar_types;
pub mod type_permissions;

use crate::metadata::resolved::error::Error;
use crate::metadata::resolved::metadata::{resolve_metadata, Metadata};

/// this is where we take the input metadata and attempt to resolve a working `Metadata` object
/// currently the bulk of this lives in the `resolve_metadata` function, we'll be slowly breaking
/// it up and moving it into steps here
pub fn resolve(metadata: open_dds::Metadata) -> Result<Metadata, Error> {
    let metadata_accessor: open_dds::accessor::MetadataAccessor =
        open_dds::accessor::MetadataAccessor::new(metadata);

    let graphql_config =
        graphql_config::resolve(&metadata_accessor.graphql_config, &metadata_accessor.flags)?;

    let data_connectors = data_connectors::resolve(&metadata_accessor)?;

    let data_connector_type_mappings::DataConnectorTypeMappingsOutput {
        data_connector_type_mappings,
        graphql_types,
        global_id_enabled_types,
        apollo_federation_entity_enabled_types,
        object_types,
    } = data_connector_type_mappings::resolve(&metadata_accessor, &data_connectors)?;

    let scalar_types::ScalarTypesOutput {
        scalar_types,
        graphql_types,
    } = scalar_types::resolve(&metadata_accessor, &graphql_types)?;

    let data_connector_scalar_types::DataConnectorWithScalarsOutput {
        data_connectors,
        graphql_types,
    } = data_connector_scalar_types::resolve(
        &metadata_accessor,
        &data_connectors,
        &scalar_types,
        &graphql_types,
    )?;

    let object_types_with_permissions =
        type_permissions::resolve(&metadata_accessor, &object_types)?;

    let boolean_expressions::BooleanExpressionsOutput {
        boolean_expression_types,
        graphql_types,
    } = boolean_expressions::resolve(
        &metadata_accessor,
        &data_connectors,
        &data_connector_type_mappings,
        &object_types_with_permissions,
        &scalar_types,
        &graphql_types,
        &graphql_config,
    )?;

    let models::ModelsOutput {
        models,
        global_id_enabled_types,
        apollo_federation_entity_enabled_types,
        graphql_types: _graphql_types,
    } = models::resolve(
        &metadata_accessor,
        &data_connectors,
        &data_connector_type_mappings,
        &graphql_types,
        &global_id_enabled_types,
        &apollo_federation_entity_enabled_types,
        &object_types_with_permissions,
        &scalar_types,
        &boolean_expression_types,
        &graphql_config,
    )?;

    let commands = commands::resolve(
        &metadata_accessor,
        &data_connectors,
        &data_connector_type_mappings,
        &object_types_with_permissions,
        &scalar_types,
        &boolean_expression_types,
    )?;

    resolve_metadata(
        &metadata_accessor,
        &graphql_config,
        global_id_enabled_types,
        apollo_federation_entity_enabled_types,
        &data_connector_type_mappings,
        object_types_with_permissions,
        &scalar_types,
        &boolean_expression_types,
        &data_connectors,
        models,
        commands,
    )
}