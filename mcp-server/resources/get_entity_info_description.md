This request allows you to get the detailed information about an Entity with it's ID. You will get the name, description, other attributes, inbound relations and outbound relations of the Entity.

The id for San Francisco is: 3qayfdjYyPv1dAYf8gPL5r

ToolCall> get_entity_info("3qayfdjYyPv1dAYf8gPL5r")
ToolResult>
```
{
  "all_attributes": [
    {
      "attribute_name": "Description",
      "attribute_value": "A vibrant city known for its iconic Golden Gate Bridge, steep rolling hills, historic cable cars, and a rich cultural tapestry including diverse neighborhoods like the Castro and the Mission District."
    },
    {
      "attribute_name": "Name",
      "attribute_value": "San Francisco"
    }
  ],
  "id": "3qayfdjYyPv1dAYf8gPL5r",
  "inbound_relations": [
    {
      "id": "NAMA1uDMzBQTvPYV9N92BV",
      "name": "SF Mayor Lurie launching police task force to counter crime in core downtown areas",
      "relation_id": "8ESicJHiNJ28VGL5u34A5q",
      "relation_type": "Related spaces"
    },
    {
      "id": "6wAoNdGVbweKi2JRPZP4bX",
      "name": "San Francisco Independent Film Festival",
      "relation_id": "TH5Tu5Y5nacvREvAQRvcR2",
      "relation_type": "Related spaces"
    },
    {
      "id": "8VCHYDURDStwuTCUBjWLQa",
      "name": "Product Engineer at Geo",
      "relation_id": "KPTqdNpCusxfM37KbKPX8w",
      "relation_type": "Related spaces"
    }, ...
  ],
  "outbound_relations": [
    {
      "id": "7gzF671tq5JTZ13naG4tnr",
      "name": "Space",
      "relation_id": "WUZCXE1UGRtxdNQpGug8Tf",
      "relation_type": "Types"
    },
    {
      "id": "CUoEazCD7EmzXPTFFY8gGY",
      "name": "No name",
      "relation_id": "5WeSkkE1XXvGJGmXj9VUQ8",
      "relation_type": "Cover"
    },
    {
      "id": "D6Wy4bdtdoUrG3PDZceHr",
      "name": "City",
      "relation_id": "ARMj8fjJtdCwbtZa1f3jwe",
      "relation_type": "Types"
    }, ...
  ]
}
```

Any of the given field can be further queried by using get_entity_info with that id since all information in the Knowledge Graph(KG) is an Entity.
