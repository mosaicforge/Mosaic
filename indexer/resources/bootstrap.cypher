// CREATE CONSTRAINT ON (n) ASSERT n.id IS UNIQUE
// CREATE CONSTRAINT ON (n) ASSERT exists(n.id)

// Attributes
CREATE (attr_type:Type { 
    name: 'Attribute',
    id: '808a04ceb21c4d888ad12e240613e5ca'
})
CREATE (val_type_attr:Attribute {
    name: 'Value Type',
    id: 'ee26ef23f7f14eb6b7423b0fa38c1fd8',
    description: 'The Type a Triple for this Attribute should have. There are Entities for the Native Types.'
})

// Native Types
CREATE (text_type:Type { name: 'Text', id: '9edb6fcce4544aa5861139d7f024c010' })
CREATE (number_type:Type { name: 'Number', id: '9b597aaec31c46c88565a370da0c2a65' })
CREATE (entity_type:Type { name: 'Entity', id: '828558cf9fbf432186eb9dbacaae3c3e' })
CREATE (checkbox_type:Type { name: 'Checkbox', id: '7aa4792eeacd41868272fa7fc18298ac' })
CREATE (uri_type:Type { name: 'URI', id: '283127c96142468492ed90b0ebc7f29a' })
CREATE (time_type:Type { name: 'Time', id: '167664f668f840e1976b20bd16ed8d47' })
CREATE (geoloc_type:Type { name: 'Geo location', id: 'df250d17e364413d97792ddaae841e34' })

// Relations
CREATE (:Type { name: 'Relation', id: 'c167ef23fb2a40449ed945123ce7d2a9' })
CREATE (from_ent_attr:Attribute { 
    name: 'From entity', 
    id: 'c43b537bcff742718822717fdf2c9c01',
    description: 'The Entity ID this relation is from.'
})
CREATE (to_ent_attr:Attribute { 
    name: 'To entity', 
    id: 'c1f4cb6fece44c3ca447ab005b756972',
    description: 'The Entity ID this relation is pointing to.'
})
CREATE (:Attribute { 
    name: 'To space', 
    id: '44264108cac144899c3532c8f96504e5',
    description: 'An optional Space ID if the relation is meant to point to a specific space.'
})
CREATE (:Attribute { 
    name: 'Index', 
    id: 'ede47e6930b044998ea4aafbda449609',
    description: 'An alphanumeric fractional index describing the relative position of this item.'
})
CREATE (:Type { name: 'Relation Type', id: '14611456b4664cab920d2245f59ce828' })
CREATE (:Attribute { name: 'Relation value type', id: 'cfa6a2f5151f43bfa684f7f0228f63ff' })

// Entities
CREATE (name_attr:Attribute {
    name: "Name", 
    id: "a126ca530c8e48d5b88882c734c38935", 
    description: "The name that shows up wherever this entity is referenced."
})

CREATE (type_attr:Attribute {
    name: "Types", 
    id: "8f151ba4de204e3c9cb499ddf96f48f1",
    description: "The one or many Types that this entity has."
})


CREATE (desc_attr:Attribute {
    name: "Description", 
    id: "9b1f76ff9711404c861e59dc3fa7d037",
    description: "A short description that is often shown in previews"
})

CREATE (cover_attr:Attribute {
    name: "Cover", 
    id: "34f535072e6b42c5a84443981a77cfa2",
    description: 'A banner style image that is wide and conveys the entity.'
})

CREATE (cont_attr:Attribute {
    name: "Content", 
    id: "beaba5cba67741a8b35377030613fc70",
    description: 'The blocks of rich content associated with this entity.'
})

// Types
// CREATE (type_type:Type { name: 'Type', id: '8f151ba4de204e3c9cb499ddf96f48f1' })
// From geo browser
CREATE (type_type:Type { name: 'Type', id: 'd7ab40920ab5441e88c35c27952de773' })

CREATE (:Attribute {name: 'Attributes', id: '01412f8381894ab1836565c7fd358cc1'})
CREATE (:Attribute {name: 'Relation type', id: 'd747a35a6aa14f468f76e6c2064c7036'})

CREATE (:Type {name: 'Space', id: '362c1dbddc6444bba3c4652f38a642d7'})

CREATE (:Type {name: 'Collection', id: 'c373a33052df47b3a6d2df552bda4b44'})

CREATE (:Type {name: 'Person', id: '912c4d9504fc4d3cbc84382b3a1904d1'})
CREATE (:Type {name: 'Project', id: 'cb9d261d456b4eaf87e51e9faa441867'})

CREATE (:Type {name: 'Image', id: 'ba4e41460010499da0a3caaa7f579d0e'})