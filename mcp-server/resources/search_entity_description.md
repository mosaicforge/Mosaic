This request allows you to search by name for the corresponding Entity in the Knowledge Graph. This will give back the most relevant Entities.

ToolCall> search_entity("San Francisco")
ToolResult> 
```
[
{
  "description": "A vibrant city known for its iconic Golden Gate Bridge, steep rolling hills, historic cable cars, and a rich cultural tapestry including diverse neighborhoods like the Castro and the Mission District.",
  "id": "3qayfdjYyPv1dAYf8gPL5r",
  "name": "San Francisco"
},
{
  "description": null,
  "id": "W5ZEpuy3Tij1XSXtJLruQ5",
  "name": "SF Bay Area"
},
{
  "description": null,
  "id": "RHoJT3hNVaw7m5fLLtZ8WQ",
  "name": "California"
},
{
  "description": null,
  "id": "Sh1qtjr4i92ZD6YGPeu5a2",
  "name": "Abundant housing in San Francisco"
},
{
  "description": null,
  "id": "UqLf9fTVKHkDs3LzP9zHpH",
  "name": "Public safety in San Francisco"
},
{
  "description": null,
  "id": "BeyiZ6oLqLMaSXiG41Yxtf",
  "name": "City"
},
{
  "description": null,
  "id": "D6Wy4bdtdoUrG3PDZceHr",
  "name": "City"
},
{
  "description": null,
  "id": "JWVrgUXmjS75PqNX2hry5q",
  "name": "Clean streets in San Francisco"
}
]
```

These Entities can be further queried using their id to get more information.
