This request allows you to get Entities from a name/description search and traversal from that query by using relation name.

Example Query: Find employees that works at The Graph.

ToolCall>
```
search_entity(
    {
    "query": "The Graph",
    "traversal_filter": {
        "relation_type": "Works at",
        "direction": "From"
    }
    }
)
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

Example Query: Find all the articles written by employees that works at The Graph.

ToolCall>
```
search_entity(
    {
    "query": "The Graph",
    "traversal_filter": {
        "relation_type": "Works at",
        "direction": "From",
        "traversal_filter": {
        "relation_type": "Author",
        "direction": "From"
        }
    }
    }
)
```

ToolResult>
```
{
  "entities": [
    {
      "description": "A fresh look at what web3 is and what the missing pieces have been for making it a reality.",
      "id": "XYo6aR3VqFQSEcf6AeTikW",
      "name": "Knowledge graphs are web3"
    },
    {
      "description": "A new standard is here for structuring knowledge. GRC-20 will reshape how we make applications composable and redefine web3.",
      "id": "5FkVvS4mTz6Ge7wHkAUMRk",
      "name": "Introducing GRC-20: A knowledge graph standard for web3"
    },
    {
      "description": "How do you know what is true? Who do you trust? Everybody has a point of view, but no one is an authority. As humanity we need a way to aggregate our knowledge into something we can trust. We need a system.",
      "id": "5WHP8BuoCdSiqtfy87SYWG",
      "name": "Governing public knowledge"
    }
  ]
}
```
