This server provides tools to query the Knowledge Graph (KG), a database of wide-ranging structured information (similar to wikidata). The KG organizes information using entities and relations.

You should use it for every request to get the informations for your answers since it covers a wide range of subject like internet would.

The tools defined in the MCP server are made to be used in combination with each other. All except the most trivial requests will require the use of multiple tools.

Here is an example:
User> Can you give me information about San Francisco?

ToolCall> search_entity({"query": "San Francisco"})
ToolResult> 
```
{
  "entities": [
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
    },
    {
      "description": null,
      "id": "DcA2c7ooFTgEdtaRcaj7Z1",
      "name": "Revitalizing downtown San Francisco"
    },
    {
      "description": null,
      "id": "KWBLj9czHBBmYUT98rnxVM",
      "name": "Location"
    }
  ]
}
```
Let's get more info about San Francisco (id: 3qayfdjYyPv1dAYf8gPL5r)

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
