use std::collections::HashMap;

use futures::{stream, StreamExt, TryStreamExt};
use kg_node::kg::entity::{Entity, EntityNode};
use kg_node::system_ids;
use swc::config::SourceMapsConfig;
use swc::PrintArgs;
use swc_common::{sync::Lrc, SourceMap, Span};
use swc_core::{quote, quote_expr};
use swc_ecma_ast::{
    Constructor, Decl, EsVersion, ExportDecl, Module, ModuleDecl, ModuleItem, TsArrayType,
    TsEntityName, TsKeywordType, TsType, TsTypeParamInstantiation, TsTypeRef,
};
use swc_ecma_ast::{Program, TsKeywordTypeKind};
use swc_ecma_codegen::Config;
use utils::{assign_this, class, class_prop, constructor, ident, method, param};

pub mod utils;

pub fn ts_type_from_value_type(value_type: &Entity) -> TsType {
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
            type_name: TsEntityName::Ident(ident(value_type.type_name())),
            type_params: None,
        }),
    }
}

pub fn gen_type_constructor(attributes: &Vec<&(Entity, Option<Entity>)>) -> Constructor {
    let super_constructor = vec![quote_expr!("super(id, driver)")];

    let constuctor_setters = attributes.iter().map(|(attr, _)| {
        let name = attr.attribute_name();
        Box::new(assign_this(
            name.clone(),
            quote_expr!("$name", name: Ident = name.into()),
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
                attr.attribute_name(),
                value_type
                    .as_ref()
                    .map(ts_type_from_value_type)
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

trait EntitiesExt {
    fn fix_name_collisions(self) -> Vec<Entity>;

    fn unique(self) -> Vec<Entity>;
}

impl<T: IntoIterator<Item = Entity>> EntitiesExt for T {
    fn fix_name_collisions(self) -> Vec<Entity> {
        let mut name_counts = HashMap::new();
        let entities = self.into_iter().collect::<Vec<_>>();

        for entity in &entities {
            let count = name_counts.entry(entity.name.clone()).or_insert(0);
            *count += 1;
        }

        entities
            .into_iter()
            .map(|mut entity| {
                let count = name_counts.get(&entity.name).unwrap();
                if *count > 1 {
                    entity.name = format!("{}_{}", entity.name, entity.id);
                }
                entity
            })
            .collect()
    }

    fn unique(self) -> Vec<Entity> {
        let entities = self
            .into_iter()
            .map(|entity| (entity.id.clone(), entity))
            .collect::<HashMap<_, _>>();

        entities.into_iter().map(|(_, entity)| entity).collect()
    }
}

trait EntityExt {
    fn type_name(&self) -> String;

    fn attribute_name(&self) -> String;
}

impl EntityExt for Entity {
    fn type_name(&self) -> String {
        if self.name == self.id {
            format!("_{}", self.name)
        } else {
            heck::AsUpperCamelCase(self.name.clone()).to_string()
        }
    }

    fn attribute_name(&self) -> String {
        if self.name == self.id {
            format!("_{}", self.name)
        } else {
            heck::AsLowerCamelCase(self.name.clone()).to_string()
        }
    }
}

/// Generate a TypeScript class declaration from an entity.
/// Note: The entity must be a `Type` entity.
pub async fn gen_type(entity: &Entity) -> anyhow::Result<Decl> {
    let typed_attrs = stream::iter(entity.attributes().await?.unique().fix_name_collisions())
        .then(|attr| async move {
            let value_type = attr.value_type().await?;
            Ok::<_, anyhow::Error>((attr, value_type))
        })
        .try_collect::<Vec<_>>()
        .await?;

    // Get all attributes of the type
    let attributes: Vec<&(Entity, Option<Entity>)> = typed_attrs
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
                attr.attribute_name(),
                value_type
                    .as_ref()
                    .map(ts_type_from_value_type)
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
                attr.attribute_name(),
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
        entity.type_name(),
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
        quote!("import { Entity } from './kg';" as ModuleItem),
    ];

    let types = kg
        .find_types::<EntityNode>()
        .await?
        .into_iter()
        .map(|node| Entity::from_entity(kg.clone(), node))
        .unique()
        .fix_name_collisions();

    let stmts = stream::iter(types)
        .then(|entity| async move {
            let decl = gen_type(&entity).await?;
            Ok::<_, anyhow::Error>(ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                span: Span::default(),
                decl,
            })))
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
