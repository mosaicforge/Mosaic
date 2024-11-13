// See https://github.com/geobrowser/geogenesis/blob/stream/1.0.0/packages/substream/sink/bootstrap-root.ts

use crate::kg::id::create_geo_id;
use crate::{system_ids};
use kg_core::pb::grc20;

// (entity_id, name)
const NAMES: &[(&str, &str)] = &[
    (system_ids::TYPES, "Types"),
    (system_ids::NAME, "Name"),
    (system_ids::ATTRIBUTE, "Attribute"),
    (system_ids::RELATION_TYPE_ATTRIBUTE, "Relation Types"),
    (system_ids::SPACE, "Indexed Space"),
    (system_ids::ATTRIBUTES, "Attributes"),
    (system_ids::SCHEMA_TYPE, "Type"),
    (system_ids::TEMPLATE_ATTRIBUTE, "Template"),
    (system_ids::VALUE_TYPE, "Value type"),
    (system_ids::RELATION_TYPE, "Relation Type"),
    (system_ids::COLLECTION_VALUE_TYPE, "Collection"),
    (system_ids::TEXT, "Text"),
    (system_ids::IMAGE, "Image"),
    (system_ids::IMAGE_URL_ATTRIBUTE, "Image URL"),
    (system_ids::DATE, "Date"),
    (system_ids::WEB_URL, "Web URL"),
    (system_ids::IMAGE_ATTRIBUTE, "Image"),
    (system_ids::DESCRIPTION, "Description"),
    (system_ids::SPACE_CONFIGURATION, "Space"),
    (system_ids::SOURCE_SPACE_ATTRIBUTE, "Source space"),
    (system_ids::FOREIGN_TYPES, "Foreign Types"),
    // Data blocks
    (system_ids::VIEW_TYPE, "View"),
    (system_ids::TABLE_BLOCK, "Table Block"),
    (system_ids::VIEW_ATTRIBUTE, "View"),
    (system_ids::GALLERY_VIEW, "Gallery View"),
    (system_ids::TABLE_VIEW, "Table View"),
    (system_ids::LIST_VIEW, "List View"),
    (system_ids::SHOWN_COLUMNS, "Shown Columns"),
    (system_ids::TEXT_BLOCK, "Text Block"),
    (system_ids::IMAGE_BLOCK, "Image Block"),
    (system_ids::BLOCKS, "Blocks"),
    (system_ids::PARENT_ENTITY, "Parent Entity"),
    (system_ids::FILTER, "Filter"),
    (system_ids::MARKDOWN_CONTENT, "Markdown Content"),
    (system_ids::ROW_TYPE, "Row Type"),
    (system_ids::PLACEHOLDER_IMAGE, "Placeholder Image"),
    (system_ids::PLACEHOLDER_TEXT, "Placeholder Text"),
    (system_ids::PERSON_TYPE, "Person"),
    (system_ids::AVATAR_ATTRIBUTE, "Avatar"),
    (system_ids::COVER_ATTRIBUTE, "Cover"),
    (system_ids::WALLETS_ATTRIBUTE, "Wallets"),
    (system_ids::BROADER_SPACES, "Broader Spaces"),
    (
        system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
        "Relation Value Types",
    ),
    (system_ids::COLLECTION_TYPE, "Collection"),
    (system_ids::RELATION, "Relation"),
    (system_ids::RELATION_INDEX, "Index"),
    (system_ids::RELATION_TO_ATTRIBUTE, "To entity"),
    (system_ids::RELATION_FROM_ATTRIBUTE, "From entity"),
];

// (attribute_id, value_type_id)
const ATTRIBUTES: &[(&str, &str)] = &[
    (system_ids::TYPES, system_ids::RELATION_TYPE),
    (system_ids::TEMPLATE_ATTRIBUTE, system_ids::RELATION_TYPE),
    (system_ids::ATTRIBUTES, system_ids::RELATION_TYPE),
    (
        system_ids::RELATION_TYPE_ATTRIBUTE,
        system_ids::RELATION_TYPE,
    ),
    (system_ids::VALUE_TYPE, system_ids::RELATION_TYPE),
    (system_ids::IMAGE_ATTRIBUTE, system_ids::TEXT),
    (system_ids::DESCRIPTION, system_ids::TEXT),
    (system_ids::NAME, system_ids::TEXT),
    (system_ids::SPACE, system_ids::TEXT),
    (system_ids::SOURCE_SPACE_ATTRIBUTE, system_ids::RELATION),
    // Data blocks
    (system_ids::VIEW_ATTRIBUTE, system_ids::RELATION_TYPE),
    (system_ids::FOREIGN_TYPES, system_ids::RELATION_TYPE),
    (system_ids::MARKDOWN_CONTENT, system_ids::TEXT),
    (system_ids::ROW_TYPE, system_ids::RELATION_TYPE),
    (system_ids::BLOCKS, system_ids::RELATION_TYPE),
    (system_ids::PARENT_ENTITY, system_ids::RELATION_TYPE),
    (system_ids::FILTER, system_ids::TEXT),
    (system_ids::PLACEHOLDER_IMAGE, system_ids::RELATION_TYPE),
    (system_ids::PLACEHOLDER_TEXT, system_ids::TEXT),
    (
        system_ids::RELATION_VALUE_RELATIONSHIP_TYPE,
        system_ids::RELATION_TYPE,
    ),
    (system_ids::AVATAR_ATTRIBUTE, system_ids::IMAGE),
    (system_ids::COVER_ATTRIBUTE, system_ids::IMAGE),
    (system_ids::WALLETS_ATTRIBUTE, system_ids::RELATION_TYPE),
    (system_ids::RELATION_INDEX, system_ids::TEXT),
    (system_ids::RELATION_TO_ATTRIBUTE, system_ids::RELATION_TYPE),
    (
        system_ids::RELATION_FROM_ATTRIBUTE,
        system_ids::RELATION_TYPE,
    ),
    (system_ids::IMAGE_URL_ATTRIBUTE, system_ids::WEB_URL),
    (system_ids::BROADER_SPACES, system_ids::RELATION_TYPE),
];

// These types include the default types and attributes for a given type. There might be more
// attributes on a type than are listed here if they were later added by users.
// (type_id, [attribute_id])
const TYPES: &[(&str, &[&str])] = &[
    (system_ids::SCHEMA_TYPE, &[system_ids::TEMPLATE_ATTRIBUTE]),
    (system_ids::VIEW_TYPE, &[]),
    (system_ids::TEXT, &[]),
    (system_ids::RELATION_TYPE, &[]),
    (system_ids::IMAGE, &[system_ids::IMAGE_URL_ATTRIBUTE]),
    (system_ids::DATE, &[]),
    (system_ids::WEB_URL, &[]),
    (system_ids::ATTRIBUTE, &[system_ids::VALUE_TYPE]),
    (
        system_ids::SPACE_CONFIGURATION,
        &[system_ids::FOREIGN_TYPES, system_ids::BLOCKS],
    ),
    (
        system_ids::IMAGE_BLOCK,
        &[system_ids::IMAGE_ATTRIBUTE, system_ids::PARENT_ENTITY],
    ),
    (
        system_ids::TABLE_BLOCK,
        &[system_ids::ROW_TYPE, system_ids::PARENT_ENTITY],
    ),
    (
        system_ids::TEXT_BLOCK,
        &[system_ids::MARKDOWN_CONTENT, system_ids::PARENT_ENTITY],
    ),
    (
        system_ids::PERSON_TYPE,
        &[system_ids::AVATAR_ATTRIBUTE, system_ids::COVER_ATTRIBUTE],
    ),
    (
        system_ids::RELATION,
        &[
            system_ids::RELATION_INDEX,
            system_ids::RELATION_TO_ATTRIBUTE,
            system_ids::RELATION_FROM_ATTRIBUTE,
            system_ids::RELATION_TYPE_ATTRIBUTE,
        ],
    ),
];

pub fn name_ops() -> Vec<grc20::Triple> {
    NAMES
        .iter()
        .map(|(id, name)| grc20::Triple {
            entity: id.to_string(),
            attribute: system_ids::NAME.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Text as i32,
                value: name.to_string(),
            }),
        })
        .collect()
}

// See https://github.com/geobrowser/geogenesis/blob/stream/1.0.0/packages/sdk/src/collections/create-relation.ts
pub fn create_relationship(
    from_id: &str,
    to_id: &str,
    relationship_type_id: &str,
) -> Vec<grc20::Triple> {
    let new_entity_id = create_geo_id();

    vec![
        // Type of Collection Item
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::TYPES.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Entity as i32,
                value: system_ids::RELATION.to_string(),
            }),
        },
        // Entity value for the collection itself
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::RELATION_FROM_ATTRIBUTE.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Entity as i32,
                value: from_id.to_string(),
            }),
        },
        // Entity value for the entity referenced by this collection item
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::RELATION_TO_ATTRIBUTE.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Entity as i32,
                value: to_id.to_string(),
            }),
        },
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::RELATION_INDEX.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Text as i32,
                value: "a0".to_string(),
            }),
        },
        grc20::Triple {
            entity: new_entity_id.clone(),
            attribute: system_ids::RELATION_TYPE_ATTRIBUTE.to_string(),
            value: Some(grc20::Value {
                r#type: grc20::ValueType::Entity as i32,
                value: relationship_type_id.to_string(),
            }),
        },
    ]
}

pub fn attribute_ops() -> Vec<grc20::Triple> {
    ATTRIBUTES
        .iter()
        .flat_map(|(attribute_id, _)| {
            create_relationship(attribute_id, system_ids::ATTRIBUTE, system_ids::TYPES)
        })
        .collect()
}

pub fn attribute_value_type_ops() -> Vec<grc20::Triple> {
    ATTRIBUTES
        .iter()
        .flat_map(|(attribute_id, value_type_id)| {
            create_relationship(attribute_id, value_type_id, system_ids::VALUE_TYPE)
        })
        .collect()
}

pub fn type_ops() -> Vec<grc20::Triple> {
    TYPES
        .iter()
        .flat_map(|(type_id, _)| {
            create_relationship(type_id, system_ids::SCHEMA_TYPE, system_ids::TYPES)
        })
        .collect()
}

pub fn type_schema_ops() -> Vec<grc20::Triple> {
    TYPES
        .iter()
        .flat_map(|(type_id, attributes)| {
            attributes
                .iter()
                .flat_map(|attribute_id| {
                    create_relationship(type_id, attribute_id, system_ids::ATTRIBUTES)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn bootstrap() -> Vec<grc20::Op> {
    let mut ops = vec![];

    ops.extend(name_ops());
    ops.extend(attribute_ops());
    ops.extend(attribute_value_type_ops());
    ops.extend(type_ops());
    ops.extend(type_schema_ops());

    ops.into_iter()
        .map(|op| grc20::Op {
            r#type: grc20::OpType::SetTriple as i32,
            triple: Some(op),
        })
        .collect()
}

const RELATION_TYPES: &[(&str, &[&str])] = &[
    (system_ids::TYPES, &[system_ids::ATTRIBUTES]),
    (system_ids::ATTRIBUTES, &[system_ids::VALUE_TYPE]),
];
