This request allows you to search by name for information of the relations between entities in the Knowledge Graph like works at.

ToolCall> search_relation_types("works at")
ToolResult>
```
[
  [
    {
      "attribute_name": "Name",
      "attribute_value": "Works at",
      "entity_id": "U1uCAzXsRSTP4vFwo1JwJG"
    },
    {
      "attribute_name": "Is type property",
      "attribute_value": "0",
      "entity_id": "U1uCAzXsRSTP4vFwo1JwJG"
    }
  ]
]
```

Since all the relation types are also of the type Entity. they can be queried by their id for more information.
