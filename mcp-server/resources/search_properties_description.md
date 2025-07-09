This request allows you to search by name for the ATTRIBUTES (properties) that can be used to describe an Entity.


ToolCall> search_properties("Authors")
ToolResult>
```
[
  {
    "description": null,
    "entity_id": "JzFpgguvcCaKhbQYPHsrNT",
    "name": "Authors"
  },
  {
    "description": null,
    "entity_id": "RwDfM3vUvyLwSNYv6sWhc9",
    "name": "Owners"
  },
  {
    "description": null,
    "entity_id": "Lc4JrkpMUPhNstqs7mvnc5",
    "name": "Publisher"
  }, ...
]
```

Since all the Properties are also of the type Entity. they can be queried by their id for more information.
