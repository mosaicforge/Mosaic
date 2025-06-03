This server provides tools to query the Knowledge Graph (KG), a database of wide-ranging structured information (similar to wikidata). The KG organizes information using entities and relations. Entities can have 0, 1 or many types, while relations have exactly one relation type. Both entities and relations can have properties.

Importantly, types, relation types and properties are themselves entities that can be queried. In other words, the KG contains both the property graph of the data as well as the data itself! 

The tools defined in the MCP server are made to be used in combination with each other. All except the most trivial user requests will require the use of multiple tools.

Here is an example:
User> What are the properties of the Person type?

ToolCall> search_type("person")
ToolResult> 
```
```