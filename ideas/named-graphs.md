# Named Graphs Support

## Overview

Add support for loading multiple RDF datasets into separate named graphs, enabling comparative queries across different scenarios (e.g., grid states at different timestamps).

## Use Cases

1. **Scenario Comparison** - Compare grid configurations at different points in time
2. **Before/After Analysis** - Detect changes between model versions
3. **Multi-Model Validation** - Cross-reference data from different sources
4. **Change Detection** - Find added, removed, or modified entities

## Proposed Configuration

```yaml
name: scenario-comparison

sources:
  - type: local
    path: /input/scenario_2026_03_01/
    graph: http://scenario/2026-03-01
    
  - type: local
    path: /input/scenario_2026_03_04/
    graph: http://scenario/2026-03-04

query:
  file: /input/diff-query.sparql
```

## Example Queries

### Find Removed Substations

```sparql
PREFIX cim: <http://iec.ch/TC57/2013/CIM-schema-cim16#>

SELECT ?substation ?name
WHERE {
  GRAPH <http://scenario/2026-03-01> {
    ?substation a cim:Substation .
    ?substation cim:IdentifiedObject.name ?name .
  }
  FILTER NOT EXISTS {
    GRAPH <http://scenario/2026-03-04> {
      ?substation a cim:Substation .
    }
  }
}
```

### Find Added Substations

```sparql
PREFIX cim: <http://iec.ch/TC57/2013/CIM-schema-cim16#>

SELECT ?substation ?name
WHERE {
  GRAPH <http://scenario/2026-03-04> {
    ?substation a cim:Substation .
    ?substation cim:IdentifiedObject.name ?name .
  }
  FILTER NOT EXISTS {
    GRAPH <http://scenario/2026-03-01> {
      ?substation a cim:Substation .
    }
  }
}
```

### Find Voltage Level Changes

```sparql
PREFIX cim: <http://iec.ch/TC57/2013/CIM-schema-cim16#>

SELECT ?substation ?name ?oldKV ?newKV
WHERE {
  GRAPH <http://scenario/2026-03-01> {
    ?substation cim:IdentifiedObject.name ?name .
    ?vl cim:VoltageLevel.Substation ?substation ;
        cim:VoltageLevel.BaseVoltage/cim:BaseVoltage.nominalVoltage ?oldKV .
  }
  GRAPH <http://scenario/2026-03-04> {
    ?vl2 cim:VoltageLevel.Substation ?substation ;
         cim:VoltageLevel.BaseVoltage/cim:BaseVoltage.nominalVoltage ?newKV .
  }
  FILTER(?oldKV != ?newKV)
}
```

### Merge and Tag by Source

```sparql
PREFIX cim: <http://iec.ch/TC57/2013/CIM-schema-cim16#>

SELECT ?substation ?name ?scenario
WHERE {
  {
    GRAPH <http://scenario/2026-03-01> {
      ?substation a cim:Substation ;
                  cim:IdentifiedObject.name ?name .
    }
    BIND("2026-03-01" AS ?scenario)
  }
  UNION
  {
    GRAPH <http://scenario/2026-03-04> {
      ?substation a cim:Substation ;
                  cim:IdentifiedObject.name ?name .
    }
    BIND("2026-03-04" AS ?scenario)
  }
}
ORDER BY ?substation ?scenario
```

## Implementation Notes

### Oxigraph Support

Oxigraph supports named graphs via quad storage:

```rust
use oxigraph::model::{GraphName, NamedNode, Quad};

// Load into named graph
let graph = GraphName::NamedNode(NamedNode::new("http://scenario/2026-03-01")?);
store.load_from_reader(RdfFormat::RdfXml, reader, Some(graph))?;
```

### Required Changes

1. **Config Schema** - Add optional `graph` field to source definitions
2. **Source Structs** - Include graph URI in `FetchedFile`
3. **SPARQL Engine** - Use `store.load_from_reader()` with graph parameter
4. **Metadata** - Track files per graph in output metadata

### Output Metadata Extension

```yaml
graphs:
  - uri: http://scenario/2026-03-01
    files_loaded: 14
    triples_loaded: 341304
  - uri: http://scenario/2026-03-04
    files_loaded: 14
    triples_loaded: 342156
```

## References

- [W3C SPARQL 1.1 Graph Store](https://www.w3.org/TR/sparql11-http-rdf-update/)
- [Oxigraph Named Graphs](https://docs.rs/oxigraph/latest/oxigraph/store/struct.Store.html)
- [RDF 1.1 Concepts - Graphs](https://www.w3.org/TR/rdf11-concepts/#section-dataset)
