This request allows you to search by name for a basic type of the Knowledge Graph(KG) like Person or Event. This will give back the type with it's name, id and description. 

ToolCall> search_type("University")
ToolResult>
```
[
  [
    {
      "attribute_name": "Description",
      "attribute_value": "An institution of higher education offering undergraduate and graduate degrees, research opportunities, and specialized academic programs.",
      "entity_id": "L8iozarUyS8bkcUiS6kPqV"
    },
    {
      "attribute_name": "Name",
      "attribute_value": "University",
      "entity_id": "L8iozarUyS8bkcUiS6kPqV"
    }
  ],
  [
    {
      "attribute_name": "Description",
      "attribute_value": "An educational institution where students acquire knowledge, skills, and credentials through structured learning programs.",
      "entity_id": "M89C7wwdJVaCW9rAVQpJbY"
    },
    {
      "attribute_name": "Name",
      "attribute_value": "School",
      "entity_id": "M89C7wwdJVaCW9rAVQpJbY"
    }
  ]
]
```


Since all the types are also of the type Entity. they can be queried by their id for more information.
