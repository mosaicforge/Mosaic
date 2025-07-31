This request allows you to search by name for information of the relations between entities in the Knowledge Graph like works at.

ToolCall> search_relation_types("works at")
ToolResult>
```
[
  {
    "description": null,
    "entity_id": "U1uCAzXsRSTP4vFwo1JwJG",
    "name": "Works at"
  },
  {
    "description": "A project that someone worked at in the past. Details about the role can be added as properties on the relation.",
    "entity_id": "8fvqALeBDwEExJsDeTcvnV",
    "name": "Worked at"
  },
  {
    "description": "The supervisor to this position. In the case of a clerkship, the supervising judge.",
    "entity_id": "WnzSw9CWE7mtgwRokF8Qxh",
    "name": "Supervisor"
  },
  {
    "description": null,
    "entity_id": "Gri4x41WSPUtpwG8BzhTpa",
    "name": "Tasks"
  }, ...
]
```

Since all the relation types are also of the type Entity. they can be queried by their id for more information.
