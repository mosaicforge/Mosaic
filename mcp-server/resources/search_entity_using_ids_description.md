This request allows you to get the Entities from a name/description search and traversal from that query if needed.


Example Query: Can you give me information about San Francisco?

ToolCall>
```
search_entity_using_ids({
"query": "San Francisco"
})
```
Tool Result>
```
{
  [
    {
      "description": "A vibrant city known for its iconic Golden Gate Bridge, steep rolling hills, historic cable cars, and a rich cultural tapestry including diverse neighborhoods like the Castro and the Mission District.",
      "entity_id": "3qayfdjYyPv1dAYf8gPL5r",
      "name": "San Francisco"
    },
    {
      "description": null,
      "entity_id": "W5ZEpuy3Tij1XSXtJLruQ5",
      "name": "SF Bay Area"
    },
    {
      "description": null,
      "entity_id": "RHoJT3hNVaw7m5fLLtZ8WQ",
      "name": "California"
    }
  ]
}
```

Another Query: Give me the employees that work at The Graph?

Work_at id: U1uCAzXsRSTP4vFwo1JwJG
ToolCall>
```
search_entity_using_ids({
"query": "The Graph",
"traversal_filter": {
  "relation_type_id": "U1uCAzXsRSTP4vFwo1JwJG",
  "direction": "From"
}
})
```
ToolResult>
```
{
  "entities": [
    {
      "description": "Founder & CEO of Geo. Cofounder of The Graph, Edge & Node, House of Web3. Building a vibrant decentralized future.",
      "id": "9HsfMWYHr9suYdMrtssqiX",
      "name": "Yaniv Tal"
    },
    {
      "description": "Developer Relations Engineer",
      "id": "22MGz47c9WHtRiHuSEPkcG",
      "name": "Kevin Jones"
    },
    {
      "description": "Description will go here",
      "id": "JYTfEcdmdjiNzBg469gE83",
      "name": "Pedro Diogo"
    }
  ]
}
```
