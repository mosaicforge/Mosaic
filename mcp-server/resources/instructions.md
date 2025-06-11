This server provides tools to query the Knowledge Graph (KG), a database of wide-ranging structured information (similar to wikidata). The KG organizes information using entities and relations.

You should use it for every request to get the informations for your answers since it covers a wide range of subject like internet would.

The tools defined in the MCP server are made to be used in combination with each other. All except the most trivial requests will require the use of multiple tools.

Here is an example:
User> Can you give me information about San Francisco?

ToolCall> search_entity("San Francisco")
ToolResult> 
```
[{"description":"A vibrant city known for its iconic Golden Gate Bridge, steep rolling hills, historic cable cars, and a rich cultural tapestry including diverse neighborhoods like the Castro and the Mission District.","id":"3qayfdjYyPv1dAYf8gPL5r","name":"San Francisco"},
{"description":null,"id":"W5ZEpuy3Tij1XSXtJLruQ5","name":"SF Bay Area"},
{"description":null,"id":"RHoJT3hNVaw7m5fLLtZ8WQ","name":"California"},
{"description":null,"id":"Sh1qtjr4i92ZD6YGPeu5a2","name":"Abundant housing in San Francisco"}]
```
Let's get more info about San Francisco (id: 3qayfdjYyPv1dAYf8gPL5r)

ToolCall> get_entity_info("3qayfdjYyPv1dAYf8gPL5r")
ToolResult> 
```
{
  "all_attributes": [
    {
      "text": "{\"attribute_name\":\"Description\",\"attribute_value\":\"A vibrant city known for its iconic Golden Gate Bridge, steep rolling hills, historic cable cars, and a rich cultural tapestry including diverse neighborhoods like the Castro and the Mission District.\"}",
      "type": "text"
    },
    {
      "text": "{\"attribute_name\":\"Name\",\"attribute_value\":\"San Francisco\"}",
      "type": "text"
    }
  ],
  "description": "A vibrant city known for its iconic Golden Gate Bridge, steep rolling hills, historic cable cars, and a rich cultural tapestry including diverse neighborhoods like the Castro and the Mission District.",
  "id": "3qayfdjYyPv1dAYf8gPL5r",
  "inbound_relations": [
    {
      "text": "{\"from_id\":\"NAMA1uDMzBQTvPYV9N92BV\",\"relation_id\":\"8ESicJHiNJ28VGL5u34A5q\",\"relation_type\":\"Related spaces\",\"to_id\":\"3qayfdjYyPv1dAYf8gPL5r\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"6wAoNdGVbweKi2JRPZP4bX\",\"relation_id\":\"TH5Tu5Y5nacvREvAQRvcR2\",\"relation_type\":\"Related spaces\",\"to_id\":\"3qayfdjYyPv1dAYf8gPL5r\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"8VCHYDURDStwuTCUBjWLQa\",\"relation_id\":\"KPTqdNpCusxfM37KbKPX8w\",\"relation_type\":\"Related spaces\",\"to_id\":\"3qayfdjYyPv1dAYf8gPL5r\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"NcQ3h9jeJSavVd8iFsUxvD\",\"relation_id\":\"AqpNtJ3XxaY4fqRCyoXbdt\",\"relation_type\":\"Cities\",\"to_id\":\"3qayfdjYyPv1dAYf8gPL5r\"}",
      "type": "text"
    }
  ],
  "name": "San Francisco",
  "outbound_relations": [
    {
      "text": "{\"from_id\":\"3qayfdjYyPv1dAYf8gPL5r\",\"relation_id\":\"5WeSkkE1XXvGJGmXj9VUQ8\",\"relation_type\":\"Cover\",\"to_id\":\"CUoEazCD7EmzXPTFFY8gGY\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"3qayfdjYyPv1dAYf8gPL5r\",\"relation_id\":\"WUZCXE1UGRtxdNQpGug8Tf\",\"relation_type\":\"Types\",\"to_id\":\"7gzF671tq5JTZ13naG4tnr\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"3qayfdjYyPv1dAYf8gPL5r\",\"relation_id\":\"ARMj8fjJtdCwbtZa1f3jwe\",\"relation_type\":\"Types\",\"to_id\":\"D6Wy4bdtdoUrG3PDZceHr\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"3qayfdjYyPv1dAYf8gPL5r\",\"relation_id\":\"V1ikGW9riu7dAP8rMgZq3u\",\"relation_type\":\"Blocks\",\"to_id\":\"AhidiWYnQ8fAbHqfzdU74k\"}",
      "type": "text"
    }
  ],
  "types": [
    "D6Wy4bdtdoUrG3PDZceHr",
    "Qu6vfQq68ecZ4PkihJ4nZN",
    "7gzF671tq5JTZ13naG4tnr"
  ]
}
```
