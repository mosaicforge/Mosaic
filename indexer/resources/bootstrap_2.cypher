// ================================================================
// Attributes
// ================================================================
// Attributes of the Attribute Type
CREATE (attr_value_type:Attribute {
    name: 'Value Type',
    id: 'ee26ef23f7f14eb6b7423b0fa38c1fd8',
    description: 'The Type a Triple for this Attribute should have. There are Entities for the Native Types.'
})
// Attributes of the Entity Type
CREATE (attr_name:Attribute {
    name: "Name", 
    id: "a126ca530c8e48d5b88882c734c38935", 
    description: "The name that shows up wherever this entity is referenced."
})
// CREATE (attr_type:Attribute {
//     name: "Type", 
//     id: "8f151ba4de204e3c9cb499ddf96f48f1",
//     description: "The one or many Types that this entity has."
// })
CREATE (attr_desc:Attribute {
    name: "Description", 
    id: "9b1f76ff9711404c861e59dc3fa7d037",
    description: "A short description that is often shown in previews"
})
CREATE (attr_cover:Attribute {
    name: "Cover", 
    id: "34f535072e6b42c5a84443981a77cfa2",
    description: 'A banner style image that is wide and conveys the entity.'
})
// CREATE (attr_content:Attribute {
//     name: "Content", 
//     id: "beaba5cba67741a8b35377030613fc70",
//     description: 'The blocks of rich content associated with this entity.'
// })
// Attributes of the RelationType Type
// CREATE (attr_rel_value_type:Attribute {
//     name: 'Relation value type',
//     id: 'cfa6a2f5151f43bfa684f7f0228f63ff'
// })
// Attributes of the Relation Type
CREATE (attr_from_ent:Attribute { 
    name: 'From entity', 
    id: 'c43b537bcff742718822717fdf2c9c01',
    description: 'The Entity ID this relation is from.'
})
CREATE (attr_to_ent:Attribute { 
    name: 'To entity', 
    id: 'c1f4cb6fece44c3ca447ab005b756972',
    description: 'The Entity ID this relation is pointing to.'
})
CREATE (attr_to_space:Attribute { 
    name: 'To space', 
    id: '44264108cac144899c3532c8f96504e5',
    description: 'An optional Space ID if the relation is meant to point to a specific space.'
})
CREATE (attr_index:Attribute { 
    name: 'Index', 
    id: 'ede47e6930b044998ea4aafbda449609',
    description: 'An alphanumeric fractional index describing the relative position of this item.'
})
// Attributes of the Type Type
// CREATE (attr_attr:Attribute { 
//     name: 'Attribute', 
//     id: '01412f8381894ab1836565c7fd358cc1'
// })
// CREATE (attr_rel_type:Attribute { 
//     name: 'Relation type', 
//     id: 'd747a35a6aa14f468f76e6c2064c7036'
// })

// ================================================================
// Relation Types
// ================================================================
// Relations Types of the Entity Type
CREATE (rel_type_type:RelationType {
    name: "Type", 
    id: "8f151ba4de204e3c9cb499ddf96f48f1",
    description: "The one or many Types that this entity has."
})
CREATE (rel_type_content:RelationType {
    name: "Content", 
    id: "beaba5cba67741a8b35377030613fc70",
    description: 'The blocks of rich content associated with this entity.'
})
// Relations Types of the Type Type
CREATE (rel_type_attr:RelationType { 
    name: 'Attribute', 
    id: '01412f8381894ab1836565c7fd358cc1'
})
CREATE (rel_type_rel_type:RelationType { 
    name: 'Relation type', 
    id: 'd747a35a6aa14f468f76e6c2064c7036'
})
CREATE (rel_type_rel_value_type:Attribute {
    name: 'Relation value type',
    id: 'cfa6a2f5151f43bfa684f7f0228f63ff'
})
// ================================================================
// Relations
// ================================================================

// ================================================================
// Types
// ================================================================
// Native types
CREATE (type_text:Type { 
    name: 'Text', 
    id: '9edb6fcce4544aa5861139d7f024c010'
})
CREATE (type_num:Type { 
    name: 'Number', 
    id: '9b597aaec31c46c88565a370da0c2a65'
})
CREATE (type_check:Type { 
    name: 'Checkbox', 
    id: '7aa4792eeacd41868272fa7fc18298ac'
})
CREATE (type_uri:Type { 
    name: 'URI', 
    id: '283127c96142468492ed90b0ebc7f29a'
})
CREATE (type_time:Type { 
    name: 'Time', 
    id: '167664f668f840e1976b20bd16ed8d47'
})
CREATE (type_geoloc:Type { 
    name: 'Geo location', 
    id: 'df250d17e364413d97792ddaae841e34'
})
// Entity Type
CREATE (type_ent:Type { 
    name: 'Entity', 
    id: '828558cf9fbf432186eb9dbacaae3c3e' 
})
// Attribute Type
CREATE (type_attr:Type { 
    name: 'Attribute',
    id: '808a04ceb21c4d888ad12e240613e5ca'
})
// Relation Type
CREATE (type_rel:Type { 
    name: 'Relation', 
    id: 'c167ef23fb2a40449ed945123ce7d2a9'
})
// RelationType Type
CREATE (type_rel_type:Type { 
    name: 'Relation Type', 
    id: '14611456b4664cab920d2245f59ce828'
})
// Type Type
CREATE (type_type:Type { 
    name: 'Type', 
    id: '8f151ba4de204e3c9cb499ddf96f48f1'
})
// Media Types
CREATE (type_image:Type { 
    name: 'Image', 
    id: 'ba4e41460010499da0a3caaa7f579d0e'
})
CREATE (type_block:Type { 
    name: 'Block', 
    id: 'block'
})
CREATE (type_space:Type {
    name: 'Space',
    id: '362c1dbddc6444bba3c4652f38a642d7'
})

// ================================================================
// Add Relations
// ================================================================
// Type of Type
CREATE (type_type)-[:TYPE]->(type_type)

// Type of Attribute
CREATE (type_attr)-[:TYPE]->(type_type)

// Type of Entity
CREATE (type_ent)-[:TYPE]->(type_type)

// Type of Relation
CREATE (type_rel)-[:TYPE]->(type_type)

// Type of RelationType
CREATE (type_rel_type)-[:TYPE]->(type_type)

// Attributes and relations of the Entity Type
CREATE (type_ent)-[:ATTRIBUTE]->(attr_name)
CREATE (type_ent)-[:RELATION_TYPE]->(rel_type_type)
CREATE (type_ent)-[:ATTRIBUTE]->(attr_desc)
CREATE (type_ent)-[:ATTRIBUTE]->(attr_cover)
CREATE (type_ent)-[:RELATION_TYPE]->(rel_type_content)

// Attributes of the Type Type
CREATE (type_type)-[:RELATION_TYPE]->(rel_type_attr)
CREATE (type_type)-[:RELATION_TYPE]->(rel_type_rel_type)

// // Attributes of the Attribute Type
CREATE (type_attr)-[:ATTRIBUTE]->(attr_value_type)

// // Attributes of the Relation Type
CREATE (type_rel)-[:ATTRIBUTE]->(attr_from_ent)
CREATE (type_rel)-[:ATTRIBUTE]->(attr_to_ent)
CREATE (type_rel)-[:ATTRIBUTE]->(attr_to_space)
CREATE (type_rel)-[:ATTRIBUTE]->(attr_index)

// // Attributes of the RelationType Type
CREATE (type_rel_type)-[:RELATION_TYPE]->(rel_type_rel_value_type)

// // Value types of the Native Types
// CREATE (attr_name) -[:VALUE_TYPE]-> (type_text)
// CREATE (rel_type_type) -[:RELATION_VALUE_TYPE]-> (rel_type)
// CREATE (attr_desc) -[:VALUE_TYPE]-> (type_text)
// CREATE (attr_cover) -[:VALUE_TYPE]-> (type_image)
// CREATE (rel_content) -[:RELATION_VALUE_TYPE]-> (type_block)

// (:Type {name: "Type"}) -[:RELATION_TYPE]-> (:RelationType {name: "Attribute"}) -[:RELATION_VALUE_TYPE]-> (:Type {name: "Attribute"})

// (:Type {name: "Attribute"}) -[:ATTRIBUTE]-> (:Attribute {name: "Value type"})

// (:Type {name: "Foo"}) -[:ATTRIBUTE]-> (:Attribute {name: "Bar"}) -[:VALUE_TYPE]-> (:Type {name: "Baz"})

// ->

// (:Foo {
//     bar: Baz
// })

// (:Type {name: "Foo"}) -[:RELATION_TYPE]-> (:RelationType {name: "Bar"}) -[:RELATION_VALUE_TYPE]-> (:Type {name: "Baz"})

// -> 

// (:Foo) -[:BAR]-> (:Baz)