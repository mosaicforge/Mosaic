This request allows you to get the detailled information about an Entity with it's ID. You will get the name, description, other attributes, inbound relations and outbound relations of the Entity.

The id for San Francisco is: 3qayfdjYyPv1dAYf8gPL5r

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
    },
    {
      "text": "{\"from_id\":\"4ojV4dS1pV2tRnzXTpcMKJ\",\"relation_id\":\"3AX4j43nywT5eBRV3s6AXi\",\"relation_type\":\"Cities\",\"to_id\":\"3qayfdjYyPv1dAYf8gPL5r\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"QoakYWCuv85FVuYdSmonxr\",\"relation_id\":\"8GEF1i3LK4Z56THjE8dVku\",\"relation_type\":\"Cities\",\"to_id\":\"3qayfdjYyPv1dAYf8gPL5r\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"JuV7jLoypebzLhkma6oZoU\",\"relation_id\":\"46aBsQyBq15DimJ2i1DX4a\",\"relation_type\":\"Cities\",\"to_id\":\"3qayfdjYyPv1dAYf8gPL5r\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"RTmcYhLVmmfgUn9L3D1J3y\",\"relation_id\":\"8uYxjzkkdjskDQAeTQomvc\",\"relation_type\":\"Cities\",\"to_id\":\"3qayfdjYyPv1dAYf8gPL5r\"}",
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
    },
    {
      "text": "{\"from_id\":\"3qayfdjYyPv1dAYf8gPL5r\",\"relation_id\":\"CvGXCmGXE7ofsgZeWad28p\",\"relation_type\":\"Blocks\",\"to_id\":\"T6iKbwZ17iv4dRdR9Qw7qV\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"3qayfdjYyPv1dAYf8gPL5r\",\"relation_id\":\"Uxpsee9LoTgJqMFfAQyJP6\",\"relation_type\":\"Blocks\",\"to_id\":\"X18WRE36mjwQ7gu3LKaLJS\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"3qayfdjYyPv1dAYf8gPL5r\",\"relation_id\":\"5WMTAzCnZH9Bsevou9GQ3K\",\"relation_type\":\"Blocks\",\"to_id\":\"HeC2pygci2tnvjTt5aEnBV\"}",
      "type": "text"
    },
    {
      "text": "{\"from_id\":\"3qayfdjYyPv1dAYf8gPL5r\",\"relation_id\":\"5TmxfepRr1THMRkGWenj5G\",\"relation_type\":\"Tabs\",\"to_id\":\"5YtYFsnWq1jupvh5AjM2ni\"}",
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

Any of the given field can be further queried by using get_entity_info with that id since all information in the Knowledge Graph(KG) is an Entity.
