This request allows you to search by name for a basic type of the Knowledge Graph(KG) like Person or Event. This will give back the type with it's name, id and description. 


ToolCall> search_types("Event")
ToolResult>
```
[
    {
  "description": null,
  "id": "AaGd6UMXfNtL6U6Xx7K8Cv",
  "name": "Event",
  "types": [
    "VdTsW1mGiy1XSooJaBBLc4"
  ]
},
{
  "description": "A claim that something has happened. This type is used together with the claim type.",
  "id": "QAdjgcq9nD7Gv98vn2vrDd",
  "name": "News event",
  "types": [
    "VdTsW1mGiy1XSooJaBBLc4"
  ]
},
{
  "description": null,
  "id": "TjSP1BaHZ7QxyBcZEM8Sdt",
  "name": "Feature",
  "types": [
    "VdTsW1mGiy1XSooJaBBLc4"
  ]
},
{
  "description": "A general concept that can be used to group things of the same category together.",
  "id": "Cj7JSjWKbcdgmUjcLWNR4V",
  "name": "Topic",
  "types": [
    "VdTsW1mGiy1XSooJaBBLc4"
  ]
},
{
  "description": "Something that someone or a group can do professionally or recreationally.",
  "id": "H7NECFeRiDkbwMq74DAKk5",
  "name": "Activity",
  "types": [
    "VdTsW1mGiy1XSooJaBBLc4"
  ]
},
{
  "description": null,
  "id": "KSKxz7Ek66SfW4euxZzKsX",
  "name": "Task",
  "types": [
    "VdTsW1mGiy1XSooJaBBLc4"
  ]
}
]
```


Since all the types are also of the type Entity. they can be queried by their id for more information.
