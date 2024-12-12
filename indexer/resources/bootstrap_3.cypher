// ================================================================
// Step 0: Type Type
// ================================================================
CREATE (type_type:Type { 
    name: 'Type', 
    id: '8f151ba4de204e3c9cb499ddf96f48f1'
})
CREATE (type_type) -[:TYPE]-> (type_type)

// ================================================================
// Step 1: Attributes
// ================================================================
CREATE (type_attr:Type { 
    name: 'Attribute',
    id: '808a04ceb21c4d888ad12e240613e5ca',
})
CREATE (type_attr) -[:TYPE]-> (type_type)

CREATE (attr_val_type:Attribute {
    name: 'Value Type',
    id: 'ee26ef23f7f14eb6b7423b0fa38c1fd8',
    description: 'The Type a Triple for this Attribute should have. There are Entities for the Native Types.',
})
CREATE (type_attr) -[:ATTRIBUTE]-> (attr_val_type)

// ================================================================
// Step 2: Relation Type
// ================================================================
CREATE (type_rel:Type { 
    name: 'Relation', 
    id: 'c167ef23fb2a40449ed945123ce7d2a9'
})
CREATE (type_rel) -[:TYPE]-> (type_type)

// ================================================================
// Step 2: Relation Type Type
// ================================================================
CREATE (type_rel_type:Type { 
    name: 'Relation Type', 
    id: '14611456b4664cab920d2245f59ce828'
})
CREATE (type_rel_type) -[:TYPE]-> (type_type)

CREATE (attr_rel_value_type:)

CREATE (rel_type_rel_type:RelationType { 
    name: 'Relation Type', 
    id: '01412f8381894ab1836565c7fd358cc1'
})
CREATE (rel_type_attr:RelationType { 
    name: 'Attribute', 
    id: 'd747a35a6aa14f468f76e6c2064c7036'
})
CREATE (type_type) -[:RELATION_TYPE]-> (rel_type_rel_type)
CREATE (type_type) -[:RELATION_TYPE]-> (rel_type_attr)



// ================================================================
// Step 1
// ================================================================