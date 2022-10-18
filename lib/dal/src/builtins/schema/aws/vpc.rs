use crate::builtins::schema::aws::{AWS_NODE_COLOR, EC2_DOCS_URL, EC2_TAG_DOCS_URL};
use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::builtins::BuiltinsError;
use crate::code_generation_prototype::CodeGenerationPrototypeContext;
use crate::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use crate::prototype_context::PrototypeContext;
use crate::socket::{SocketArity, SocketEdgeKind, SocketKind};
use crate::validation::Validation;
use crate::{
    attribute::context::AttributeContextBuilder,
    func::argument::FuncArgument,
    schema::{SchemaUiMenu, SchemaVariant},
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueError, BuiltinsResult, CodeGenerationPrototype, CodeLanguage, DalContext,
    DiagramKind, ExternalProvider, Func, InternalProvider, PropKind, SchemaError, SchemaKind,
    Socket, StandardModel,
};

const INGRESS_EGRESS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html";
const SECURITY_GROUP_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-security-groups.html";
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";

const INGRESS_EGRESS_PROTOCOLS: &[&str; 3] = &["tcp", "udp", "icmp"];

pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    ingress(ctx).await?;
    egress(ctx).await?;
    security_group(ctx).await?;
    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`AWS Ingress`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
async fn ingress(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "Ingress".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup.
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let ui_menu = SchemaUiMenu::new(ctx, "Ingress", "AWS", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop Creation
    let group_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "GroupId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let protocol_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "IpProtocol",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let expected = INGRESS_EGRESS_PROTOCOLS
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>();
    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::StringInStringArray {
            value: None,
            expected,
            display_expected: true,
        },
        *protocol_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    let to_port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "ToPort",
        PropKind::Integer,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::IntegerIsBetweenTwoIntegers {
            value: None,
            lower_bound: -1,
            upper_bound: 65537,
        },
        *to_port_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    let from_port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "FromPort",
        PropKind::Integer,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::IntegerIsBetweenTwoIntegers {
            value: None,
            lower_bound: -1,
            upper_bound: 65537,
        },
        *from_port_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    let cidr_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "CidrIp",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::StringIsValidIpAddr { value: None },
        *cidr_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(AWS_REGIONS_DOCS_URL.to_string()),
    )
    .await?;

    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    let tags_map_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Map,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    let tags_map_item_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    // System Socket
    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    schema_variant.add_socket(ctx, system_socket.id()).await?;

    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    // Input Socket
    let (group_id_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Security Group ID",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (region_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    // Code Generation
    let code_generation_func_name = "si:generateAwsJSON".to_owned();
    let code_generation_func =
        Func::find_by_attr(ctx, "name", &code_generation_func_name.to_owned())
            .await?
            .pop()
            .ok_or(SchemaError::FuncNotFound(code_generation_func_name))?;

    let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
    let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
    let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
    code_generation_prototype_context.set_schema_variant_id(*schema_variant.id());

    CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Json,
        code_generation_prototype_context,
    )
    .await?;

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

    // Set Defaults
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *aws_resource_type_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["security-group-rule"],
    )
    .await?;
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *protocol_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["tcp"],
    )
    .await?;

    // Bind sockets to providers
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    let tags_map_attribute_read_context = AttributeReadContext {
        prop_id: Some(*tags_map_prop.id()),
        ..base_attribute_read_context
    };
    let tags_map_attribute_value =
        AttributeValue::find_for_context(ctx, tags_map_attribute_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                tags_map_attribute_read_context,
            ))?;
    let tags_map_item_attribute_context =
        AttributeContextBuilder::from(base_attribute_read_context)
            .set_prop_id(*tags_map_item_prop.id())
            .to_context()?;
    let name_tags_item_attribute_value_id = AttributeValue::insert_for_context(
        ctx,
        tags_map_item_attribute_context,
        *tags_map_attribute_value.id(),
        None,
        Some("Name".to_string()),
    )
    .await?;

    // Connect props to providers.

    let si_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
        .await?
        .ok_or_else(|| {
            BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
        })?;
    let name_tags_item_attribute_value =
        AttributeValue::get_by_id(ctx, &name_tags_item_attribute_value_id)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFound(
                name_tags_item_attribute_value_id,
            ))?;
    let mut name_tags_item_attribute_prototype = name_tags_item_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    name_tags_item_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    let identity_arg = FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_id)
        .await?
        .ok_or_else(|| {
            BuiltinsError::BuiltinMissingFuncArgument(
                "identity".to_string(),
                "identity".to_string(),
            )
        })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *name_tags_item_attribute_prototype.id(),
        *identity_arg.id(),
        *si_name_internal_provider.id(),
    )
    .await?;

    // Bind sockets to providers
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // region from input socket
    let region_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*region_prop.id()),
        ..base_attribute_read_context
    };
    let region_attribute_value =
        AttributeValue::find_for_context(ctx, region_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                region_attribute_value_read_context,
            ))?;
    let mut region_attribute_prototype = region_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    region_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *region_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *region_explicit_internal_provider.id(),
    )
    .await?;

    // security group id from input socket
    let group_id_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*group_id_prop.id()),
        ..base_attribute_read_context
    };
    let group_id_attribute_value =
        AttributeValue::find_for_context(ctx, group_id_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                group_id_attribute_value_read_context,
            ))?;
    let mut group_id_attribute_prototype = group_id_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    group_id_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *group_id_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *group_id_internal_provider.id(),
    )
    .await?;

    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`AWS Egress`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
async fn egress(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "Egress".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    // Variant setup.
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;
    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    let ui_menu = SchemaUiMenu::new(ctx, "Egress", "AWS", &diagram_kind).await?;
    ui_menu.set_schema(ctx, schema.id()).await?;

    // Prop Creation
    let group_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "GroupId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let protocol_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "IpProtocol",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    let expected = INGRESS_EGRESS_PROTOCOLS
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>();
    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::StringInStringArray {
            value: None,
            expected,
            display_expected: true,
        },
        *protocol_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    let from_port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "FromPort",
        PropKind::Integer,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::IntegerIsBetweenTwoIntegers {
            value: None,
            lower_bound: -1,
            upper_bound: 65537,
        },
        *from_port_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    let to_port_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "ToPort",
        PropKind::Integer,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::IntegerIsBetweenTwoIntegers {
            value: None,
            lower_bound: -1,
            upper_bound: 65537,
        },
        *to_port_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    let cidr_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "CidrIp",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(INGRESS_EGRESS_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_validation(
        ctx,
        Validation::StringIsValidIpAddr { value: None },
        *cidr_prop.id(),
        *schema.id(),
        *schema_variant.id(),
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(AWS_REGIONS_DOCS_URL.to_string()),
    )
    .await?;

    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    let tags_map_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Map,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    let tags_map_item_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    // System Socket
    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    schema_variant.add_socket(ctx, system_socket.id()).await?;

    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    // Input Socket
    let (group_id_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Security Group ID",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (region_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    // Code Generation
    let code_generation_func_name = "si:generateAwsJSON".to_owned();
    let code_generation_func =
        Func::find_by_attr(ctx, "name", &code_generation_func_name.to_owned())
            .await?
            .pop()
            .ok_or(SchemaError::FuncNotFound(code_generation_func_name))?;

    let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
    let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
    let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
    code_generation_prototype_context.set_schema_variant_id(*schema_variant.id());

    CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Json,
        code_generation_prototype_context,
    )
    .await?;

    // Wrap it up.
    schema_variant.finalize(ctx).await?;

    // Set Defaults
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *aws_resource_type_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["security-group-rule"],
    )
    .await?;
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *protocol_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["tcp"],
    )
    .await?;

    // Bind sockets to providers
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    let tags_map_attribute_read_context = AttributeReadContext {
        prop_id: Some(*tags_map_prop.id()),
        ..base_attribute_read_context
    };
    let tags_map_attribute_value =
        AttributeValue::find_for_context(ctx, tags_map_attribute_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                tags_map_attribute_read_context,
            ))?;
    let tags_map_item_attribute_context =
        AttributeContextBuilder::from(base_attribute_read_context)
            .set_prop_id(*tags_map_item_prop.id())
            .to_context()?;
    let name_tags_item_attribute_value_id = AttributeValue::insert_for_context(
        ctx,
        tags_map_item_attribute_context,
        *tags_map_attribute_value.id(),
        None,
        Some("Name".to_string()),
    )
    .await?;

    // Connect props to providers.

    let si_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
        .await?
        .ok_or_else(|| {
            BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
        })?;
    let name_tags_item_attribute_value =
        AttributeValue::get_by_id(ctx, &name_tags_item_attribute_value_id)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFound(
                name_tags_item_attribute_value_id,
            ))?;
    let mut name_tags_item_attribute_prototype = name_tags_item_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    name_tags_item_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    let identity_arg = FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_id)
        .await?
        .ok_or_else(|| {
            BuiltinsError::BuiltinMissingFuncArgument(
                "identity".to_string(),
                "identity".to_string(),
            )
        })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *name_tags_item_attribute_prototype.id(),
        *identity_arg.id(),
        *si_name_internal_provider.id(),
    )
    .await?;

    // Bind sockets to providers
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // region from input socket
    let region_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*region_prop.id()),
        ..base_attribute_read_context
    };
    let region_attribute_value =
        AttributeValue::find_for_context(ctx, region_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                region_attribute_value_read_context,
            ))?;
    let mut region_attribute_prototype = region_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    region_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *region_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *region_explicit_internal_provider.id(),
    )
    .await?;

    // security group id from input socket
    let group_id_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*group_id_prop.id()),
        ..base_attribute_read_context
    };
    let group_id_attribute_value =
        AttributeValue::find_for_context(ctx, group_id_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                group_id_attribute_value_read_context,
            ))?;
    let mut group_id_attribute_prototype = group_id_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    group_id_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *group_id_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *group_id_internal_provider.id(),
    )
    .await?;

    Ok(())
}

/// A [`Schema`](crate::Schema) migration for [`AWS Security Group`](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_SecurityGroups.html).
async fn security_group(ctx: &DalContext) -> BuiltinsResult<()> {
    let name = "Security Group".to_string();
    let mut schema =
        match BuiltinSchemaHelpers::create_schema(ctx, &name, &SchemaKind::Configuration).await? {
            Some(schema) => schema,
            None => return Ok(()),
        };

    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0").await?;
    schema_variant.set_color(ctx, Some(AWS_NODE_COLOR)).await?;

    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await?;

    let mut attribute_context_builder = AttributeContext::builder();
    attribute_context_builder
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id());

    // Diagram and UI Menu
    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    SchemaUiMenu::new(ctx, "Security Group", "AWS", &diagram_kind)
        .await?
        .set_schema(ctx, schema.id())
        .await?;

    // Prop Creation
    let security_group_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "SecurityGroupId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(SECURITY_GROUP_DOCS_URL.to_string()),
    )
    .await?;

    BuiltinSchemaHelpers::create_prop(
        ctx,
        "Description",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(SECURITY_GROUP_DOCS_URL.to_string()),
    )
    .await?;

    let group_name_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "GroupName",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(SECURITY_GROUP_DOCS_URL.to_string()),
    )
    .await?;

    let _vpc_id_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "VpcId",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(SECURITY_GROUP_DOCS_URL.to_string()),
    )
    .await?;

    let region_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "region",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(AWS_REGIONS_DOCS_URL.to_string()),
    )
    .await?;

    let tags_map_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tags",
        PropKind::Map,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    let tags_map_item_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "tag",
        PropKind::String,
        None,
        Some(*tags_map_prop.id()),
        Some(EC2_TAG_DOCS_URL.to_string()),
    )
    .await?;

    let aws_resource_type_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "awsResourceType",
        PropKind::String,
        None,
        Some(root_prop.domain_prop_id),
        Some(EC2_DOCS_URL.to_string()),
    )
    .await?;

    // Socket Creation
    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_identity_arg_id,
    ) = BuiltinSchemaHelpers::setup_identity_func(ctx).await?;

    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await?;
    schema_variant.add_socket(ctx, system_socket.id()).await?;

    let (region_explicit_internal_provider, mut input_socket) =
        InternalProvider::new_explicit_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Region",
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    input_socket.set_color(ctx, Some(0xd61e8c)).await?;

    let (security_group_id_external_provider, mut output_socket) =
        ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Security Group ID",
            None,
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            DiagramKind::Configuration,
        )
        .await?;
    output_socket.set_color(ctx, Some(0xd61e8c)).await?;

    // Code Generation
    let code_generation_func_name = "si:generateAwsJSON".to_owned();
    let code_generation_func =
        Func::find_by_attr(ctx, "name", &code_generation_func_name.to_owned())
            .await?
            .pop()
            .ok_or(SchemaError::FuncNotFound(code_generation_func_name))?;

    let code_generation_args = FuncBackendJsCodeGenerationArgs::default();
    let code_generation_args_json = serde_json::to_value(&code_generation_args)?;
    let mut code_generation_prototype_context = CodeGenerationPrototypeContext::new();
    code_generation_prototype_context.set_schema_variant_id(*schema_variant.id());

    CodeGenerationPrototype::new(
        ctx,
        *code_generation_func.id(),
        code_generation_args_json,
        CodeLanguage::Json,
        code_generation_prototype_context,
    )
    .await?;

    // Wrap it up!
    schema_variant.finalize(ctx).await?;

    // Set Defaults
    BuiltinSchemaHelpers::set_default_value_for_prop(
        ctx,
        *aws_resource_type_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        serde_json::json!["security-group"],
    )
    .await?;

    // Bind sockets to providers
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    let tags_map_attribute_read_context = AttributeReadContext {
        prop_id: Some(*tags_map_prop.id()),
        ..base_attribute_read_context
    };
    let tags_map_attribute_value =
        AttributeValue::find_for_context(ctx, tags_map_attribute_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                tags_map_attribute_read_context,
            ))?;
    let tags_map_item_attribute_context =
        AttributeContextBuilder::from(base_attribute_read_context)
            .set_prop_id(*tags_map_item_prop.id())
            .to_context()?;
    let name_tags_item_attribute_value_id = AttributeValue::insert_for_context(
        ctx,
        tags_map_item_attribute_context,
        *tags_map_attribute_value.id(),
        None,
        Some("Name".to_string()),
    )
    .await?;

    // Connect props to providers.

    let si_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
        .await?
        .ok_or_else(|| {
            BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
        })?;
    let name_tags_item_attribute_value =
        AttributeValue::get_by_id(ctx, &name_tags_item_attribute_value_id)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFound(
                name_tags_item_attribute_value_id,
            ))?;
    let mut name_tags_item_attribute_prototype = name_tags_item_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    name_tags_item_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    let identity_arg = FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_id)
        .await?
        .ok_or_else(|| {
            BuiltinsError::BuiltinMissingFuncArgument(
                "identity".to_string(),
                "identity".to_string(),
            )
        })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *name_tags_item_attribute_prototype.id(),
        *identity_arg.id(),
        *si_name_internal_provider.id(),
    )
    .await?;

    // Socket Binding
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // security_group_id to output socket
    let security_group_id_external_provider_attribute_prototype_id =
        security_group_id_external_provider
            .attribute_prototype_id()
            .ok_or_else(|| {
                BuiltinsError::MissingAttributePrototypeForExternalProvider(
                    *security_group_id_external_provider.id(),
                )
            })?;

    let security_group_id_internal_provider =
        InternalProvider::get_for_prop(ctx, *security_group_id_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*security_group_id_prop.id())
            })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *security_group_id_external_provider_attribute_prototype_id,
        identity_func_identity_arg_id,
        *security_group_id_internal_provider.id(),
    )
    .await?;

    // region from input socket
    let region_attribute_value_read_context = AttributeReadContext {
        prop_id: Some(*region_prop.id()),
        ..base_attribute_read_context
    };
    let region_attribute_value =
        AttributeValue::find_for_context(ctx, region_attribute_value_read_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                region_attribute_value_read_context,
            ))?;
    let mut region_attribute_prototype = region_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
    region_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *region_attribute_prototype.id(),
        identity_func_identity_arg_id,
        *region_explicit_internal_provider.id(),
    )
    .await?;

    // Make GroupName take the value of /root/si/name
    let group_name_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*group_name_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await?
    .ok_or(AttributeValueError::Missing)?;
    let mut group_name_attribute_proto = group_name_attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(AttributeValueError::MissingAttributePrototype)?;
    group_name_attribute_proto
        .set_func_id(ctx, identity_func_id)
        .await?;
    let si_name_prop =
        BuiltinSchemaHelpers::find_child_prop_by_name(ctx, root_prop.si_prop_id, "name").await?;
    let si_name_internal_provider = InternalProvider::get_for_prop(ctx, *si_name_prop.id())
        .await?
        .ok_or_else(|| {
            BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
        })?;
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *group_name_attribute_proto.id(),
        identity_func_identity_arg_id,
        *si_name_internal_provider.id(),
    )
    .await?;

    Ok(())
}
