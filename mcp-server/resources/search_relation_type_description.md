This request allows you to search by name for information of the relations between entities in the Knowledge Graph like works at.

ToolCall> search_relation_types("works at")
ToolResult>
```
[
    {
  "description": null,
  "id": "U1uCAzXsRSTP4vFwo1JwJG",
  "name": "Works at"
},
{
  "description": "A project that someone worked at in the past. Details about the role can be added as properties on the relation.",
  "id": "8fvqALeBDwEExJsDeTcvnV",
  "name": "Worked at"
},
{
  "description": "The supervisor to this position. In the case of a clerkship, the supervising judge.",
  "id": "WnzSw9CWE7mtgwRokF8Qxh",
  "name": "Supervisor"
},
{
  "description": null,
  "id": "Gri4x41WSPUtpwG8BzhTpa",
  "name": "Tasks"
},
{
  "description": "The judge or magistrate responsible for overseeing and deciding the case.",
  "id": "PuLfk3sFs6PkhEuf8cyBfs",
  "name": "Assigned to"
},
{
  "description": null,
  "id": "MuMLDVbHAmRjZQjhyk3HGx",
  "name": "Network"
},
{
  "description": null,
  "id": "RERshk4JoYoMC17r1qAo9J",
  "name": "From"
}
]
```

Since all the relation types are also of the type Entity. they can be queried by their id for more information.
