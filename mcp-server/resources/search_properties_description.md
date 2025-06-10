This request allows you to search by name for a basic Relation of the Knowledge Graph(KG) like Owners or Authors. This will give back the 


ToolCall> search_properties("Authors")
ToolResult>
```
[
    {
  "description": null,
  "id": "JzFpgguvcCaKhbQYPHsrNT",
  "name": "Authors",
  "types": [
    "GscJ2GELQjmLoaVrYyR3xm"
  ]
},
{
  "description": null,
  "id": "Lc4JrkpMUPhNstqs7mvnc5",
  "name": "Publisher",
  "types": [
    "GscJ2GELQjmLoaVrYyR3xm"
  ]
},
{
  "description": null,
  "id": "61dgWvCDk8QRW2yrfkHuia",
  "name": "Published in",
  "types": [
    "GscJ2GELQjmLoaVrYyR3xm"
  ]
},
{
  "description": null,
  "id": "W2aFZPy5nnU3DgdkWJCNVn",
  "name": "Person",
  "types": [
    "GscJ2GELQjmLoaVrYyR3xm"
  ]
},
{
  "description": null,
  "id": "RwDfM3vUvyLwSNYv6sWhc9",
  "name": "Owners",
  "types": [
    "GscJ2GELQjmLoaVrYyR3xm"
  ]
}
]
```

Since all the Relations are also of the type Entity. they can be queried by their id for more information.
