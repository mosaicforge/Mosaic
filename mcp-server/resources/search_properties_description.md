This request allows you to search by name for the ATTRIBUTES (properties) that can be used to describe an Entity.


ToolCall> search_properties("Authors")
ToolResult>
```
[
  [
    {
      "attribute_name": "Name",
      "attribute_value": "Authors",
      "entity_id": "JzFpgguvcCaKhbQYPHsrNT"
    }
  ],
  [
    {
      "attribute_name": "Name",
      "attribute_value": "Owners",
      "entity_id": "RwDfM3vUvyLwSNYv6sWhc9"
    }
  ]
]
```

Since all the Relations are also of the type Entity. they can be queried by their id for more information.
