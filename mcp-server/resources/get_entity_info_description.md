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
    },
    {
      "id": "NcQ3h9jeJSavVd8iFsUxvD",
      "name": "Senior Civil Engineer @ Golden Gate Bridge, Highway & Transportation District",
      "relation_id": "AqpNtJ3XxaY4fqRCyoXbdt",
      "relation_type": "Cities"
    },
    {
      "id": "4ojV4dS1pV2tRnzXTpcMKJ",
      "name": "Senior Plan Check Engineer (FT - Hybrid) @ CSG Consultants, Inc.",
      "relation_id": "3AX4j43nywT5eBRV3s6AXi",
      "relation_type": "Cities"
    },
    {
      "id": "QoakYWCuv85FVuYdSmonxr",
      "name": "Senior Civil Engineer - Land Development (FT - Hybrid) @ CSG Consultants, Inc.",
      "relation_id": "8GEF1i3LK4Z56THjE8dVku",
      "relation_type": "Cities"
    },
    {
      "id": "JuV7jLoypebzLhkma6oZoU",
      "name": "Lead Django Backend Engineer @ Textme Inc",
      "relation_id": "46aBsQyBq15DimJ2i1DX4a",
      "relation_type": "Cities"
    },
    {
      "id": "RTmcYhLVmmfgUn9L3D1J3y",
      "name": "Chief Engineer @ Wyndham Hotels & Resorts",
      "relation_id": "8uYxjzkkdjskDQAeTQomvc",
      "relation_type": "Cities"
    }
  ],
  "outbound_relations": [
    {
      "id": "CUoEazCD7EmzXPTFFY8gGY",
      "name": "No name",
      "relation_id": "5WeSkkE1XXvGJGmXj9VUQ8",
      "relation_type": "Cover"
    },
    {
      "id": "7gzF671tq5JTZ13naG4tnr",
      "name": "Space",
      "relation_id": "WUZCXE1UGRtxdNQpGug8Tf",
      "relation_type": "Types"
    },
    {
      "id": "D6Wy4bdtdoUrG3PDZceHr",
      "name": "City",
      "relation_id": "ARMj8fjJtdCwbtZa1f3jwe",
      "relation_type": "Types"
    },
    {
      "id": "AhidiWYnQ8fAbHqfzdU74k",
      "name": "Upcoming events",
      "relation_id": "V1ikGW9riu7dAP8rMgZq3u",
      "relation_type": "Blocks"
    },
    {
      "id": "T6iKbwZ17iv4dRdR9Qw7qV",
      "name": "Trending restaurants",
      "relation_id": "CvGXCmGXE7ofsgZeWad28p",
      "relation_type": "Blocks"
    },
    {
      "id": "X18WRE36mjwQ7gu3LKaLJS",
      "name": "Neighborhoods",
      "relation_id": "Uxpsee9LoTgJqMFfAQyJP6",
      "relation_type": "Blocks"
    },
    {
      "id": "HeC2pygci2tnvjTt5aEnBV",
      "name": "Top goals",
      "relation_id": "5WMTAzCnZH9Bsevou9GQ3K",
      "relation_type": "Blocks"
    },
    {
      "id": "5YtYFsnWq1jupvh5AjM2ni",
      "name": "Culture",
      "relation_id": "5TmxfepRr1THMRkGWenj5G",
      "relation_type": "Tabs"
    }
  ]
}
```

Any of the given field can be further queried by using get_entity_info with that id since all information in the Knowledge Graph(KG) is an Entity.
