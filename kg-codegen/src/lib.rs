use std::collections::HashMap;
use std::f32::consts::E;

use futures::{stream, StreamExt, TryStreamExt};
use kg_node::kg::grc20;
use kg_node::system_ids;
use swc::config::SourceMapsConfig;
use swc::PrintArgs;
use swc_common::{sync::Lrc, SourceMap, Span, SyntaxContext};
use swc_core::{quote, quote_expr};
use swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, BlockStmt, Class, ClassDecl, ClassMember,
    ClassMethod, ClassProp, Constructor, Decl, EsVersion, Expr, ExprStmt, Function, Ident,
    IdentName, MemberExpr, MemberProp, MethodKind, Module, ModuleDecl, ModuleItem, Param,
    ParamOrTsParamProp, Pat, PropName, ReturnStmt, SimpleAssignTarget, Stmt, ThisExpr, Tpl,
    TsArrayType, TsEntityName, TsExprWithTypeArgs, TsInterfaceBody, TsInterfaceDecl, TsKeywordType,
    TsPropertySignature, TsType, TsTypeAnn, TsTypeElement, TsTypeParamInstantiation, TsTypeRef,
};
use swc_ecma_ast::{Program, Script, TsKeywordTypeKind};
use swc_ecma_codegen::Config;
use utils::{assign_this, class, class_prop, constructor, ident, method, param};

pub mod utils;

pub fn ts_type_from_value_type(value_type: &grc20::Entity) -> TsType {
    match &value_type.id {
        id if id == system_ids::TEXT => TsType::TsKeywordType(TsKeywordType {
            span: Span::default(),
            kind: TsKeywordTypeKind::TsStringKeyword,
        }),
        id if id == system_ids::NUMBER => TsType::TsKeywordType(TsKeywordType {
            span: Span::default(),
            kind: TsKeywordTypeKind::TsNumberKeyword,
        }),
        id if id == system_ids::CHECKBOX => TsType::TsKeywordType(TsKeywordType {
            span: Span::default(),
            kind: TsKeywordTypeKind::TsBooleanKeyword,
        }),
        _ => TsType::TsTypeRef(TsTypeRef {
            span: Span::default(),
            type_name: TsEntityName::Ident(ident(value_type.name.clone())),
            type_params: None,
        }),
    }
}

pub fn gen_type_constructor(
    attributes: &Vec<&(grc20::Entity, Option<grc20::Entity>)>,
) -> Constructor {
    let super_constructor = vec![quote_expr!("this.super(id, driver)")];

    let constuctor_setters = attributes.iter().map(|(attr, _)| {
        let name = heck::AsLowerCamelCase(attr.name.clone()).to_string();
        Box::new(assign_this(
            name.clone(),
            quote_expr!("$name", name: Expr = name.into()),
        ))
    });

    let super_params = vec![
        param(
            "id",
            TsType::TsKeywordType(TsKeywordType {
                span: Span::default(),
                kind: TsKeywordTypeKind::TsStringKeyword,
            }),
        ),
        param(
            "driver",
            TsType::TsTypeRef(TsTypeRef {
                span: Span::default(),
                type_name: TsEntityName::Ident(ident("Driver")),
                type_params: None,
            }),
        ),
    ];

    let current_params = attributes
        .iter()
        .map(|(attr, value_type)| {
            param(
                heck::AsLowerCamelCase(attr.name.clone()),
                value_type
                    .as_ref()
                    .map(|value_type| ts_type_from_value_type(&value_type))
                    .unwrap_or(TsType::TsKeywordType(TsKeywordType {
                        span: Span::default(),
                        kind: TsKeywordTypeKind::TsAnyKeyword,
                    })),
            )
        })
        .collect::<Vec<_>>();

    constructor(
        super_params.into_iter().chain(current_params).collect(),
        Some(
            super_constructor
                .into_iter()
                .chain(constuctor_setters)
                .collect(),
        ),
    )
}

/// Generate a TypeScript class declaration from an entity.
/// Note: The entity must be a `Type` entity.
pub async fn gen_type(entity: &grc20::Entity) -> anyhow::Result<Decl> {
    let typed_attrs = stream::iter(entity.attributes().await?)
        .then(|attr| async move {
            let value_type = attr.value_type().await?;
            Ok::<_, anyhow::Error>((attr, value_type))
        })
        .try_collect::<Vec<_>>()
        .await?;

    // Get all attributes of the type
    let attributes: Vec<&(grc20::Entity, Option<grc20::Entity>)> = typed_attrs
        .iter()
        .filter(|(_, value_type)| match value_type {
            Some(value_type) if value_type.id == system_ids::RELATION_TYPE => false,
            _ => true,
        })
        .collect();

    let attribute_class_props = attributes
        .iter()
        .map(|(attr, value_type)| {
            class_prop(
                heck::AsLowerCamelCase(attr.name.clone()),
                value_type
                    .as_ref()
                    .map(|value_type| ts_type_from_value_type(&value_type))
                    .unwrap_or(TsType::TsKeywordType(TsKeywordType {
                        span: Span::default(),
                        kind: TsKeywordTypeKind::TsAnyKeyword,
                    })),
            )
        })
        .collect();

    // Get all relations of the type
    let relation_methods = typed_attrs.iter()
        .filter(|(_, value_type)| match value_type {
            Some(value_type) if value_type.id == system_ids::RELATION_TYPE => true,
            _ => false,
        })
        .map(|(attr, _)| {
            let neo4j_query = format!("MATCH ({{id: $id}}) -[r:`{}`]-> (n) RETURN n", attr.id);
            method(
                heck::AsLowerCamelCase(attr.name.clone()),
                vec![],
                None,
                Some(quote_expr!(r#"this.query($query, {id: this.id})"#, query: Expr = neo4j_query.into())),
                true,
                Some(TsType::TsTypeRef(TsTypeRef {
                    span: Span::default(),
                    type_name: TsEntityName::Ident(ident("Promise")),
                    type_params: Some(Box::new(TsTypeParamInstantiation {
                        span: Span::default(),
                        params: vec![Box::new(TsType::TsArrayType(TsArrayType {
                            span: Span::default(),
                            elem_type: Box::new(TsType::TsTypeRef(TsTypeRef {
                                span: Span::default(),
                                type_name: TsEntityName::Ident(ident("Node")),
                                type_params: None,
                            })),
                        }))],
                    })),
                })),
            )}
        )
        .collect();

    Ok(Decl::Class(class(
        heck::AsUpperCamelCase(entity.name.clone()),
        attribute_class_props,
        Some(gen_type_constructor(&attributes)),
        relation_methods,
        vec![],
        Some(ident("Entity")),
    )))
}

/// Generate a TypeScript module containing class definitions from all types in the knowledge graph.
pub async fn gen_types(kg: &kg_node::kg::Client) -> anyhow::Result<Program> {
    let import_stmts = vec![
        quote!("import { Driver, Node } from 'neo4j-driver';" as ModuleItem),
        quote!("import { Entity } from './entity';" as ModuleItem),
    ];

    let stmts = stream::iter(kg.find_types::<grc20::EntityNode>().await?)
        .then(|node| async move {
            let entity = grc20::Entity::from_entity(kg.clone(), node);
            let def = gen_type(&entity).await?;
            Ok::<_, anyhow::Error>(ModuleItem::Stmt(Stmt::Decl(def)))
        })
        .try_collect::<Vec<_>>()
        .await?;

    Ok(Program::Module(Module {
        span: Span::default(),
        body: import_stmts.into_iter().chain(stmts).collect(),
        shebang: None,
    }))
}

/// Generate and render TypeScript code from the knowledge graph.
pub async fn codegen(kg: &kg_node::kg::Client) -> anyhow::Result<String> {
    let cm: Lrc<SourceMap> = Default::default();
    let compiler = swc::Compiler::new(cm.clone());

    let ast_printed = compiler.print(
        &gen_types(kg).await?,
        PrintArgs {
            source_root: None,
            source_file_name: None,
            output_path: None,
            inline_sources_content: false,
            source_map: SourceMapsConfig::default(),
            source_map_names: &HashMap::default(),
            orig: None,
            comments: None,
            emit_source_map_columns: false,
            preamble: "",
            codegen_config: Config::default().with_target(EsVersion::latest()),
            output: None,
        },
    )?;

    Ok(ast_printed.code)
}
