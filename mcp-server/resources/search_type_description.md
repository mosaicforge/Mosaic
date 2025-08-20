This request allows you to search by name for a basic type of the Knowledge Graph(KG) like Person or Event. This will give back the type with it's name, id and description. 

ToolCall> search_types("University")
ToolResult>
```
[
  {
    "description": "An institution of higher education offering undergraduate and graduate degrees, research opportunities, and specialized academic programs.",
    "entity_id": "L8iozarUyS8bkcUiS6kPqV",
    "name": "University"
  },
  {
    "description": "An educational institution where students acquire knowledge, skills, and credentials through structured learning programs.",
    "entity_id": "M89C7wwdJVaCW9rAVQpJbY",
    "name": "School"
  },
  {
    "description": null,
    "entity_id": "ExCjm3rzYVfpMRwDchdrE",
    "name": "Academic field"
  }, ...
]
```

Since all the types are also of the type Entity. they can be queried by their id for more information.
