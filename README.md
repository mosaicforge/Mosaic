<h1 align="center">
 Mosaic
</h1>
<p align="center">
  <img width="400" alt="mosaic" src="https://pbs.twimg.com/profile_images/1890229652326612992/SsN44tyU_400x400.jpg"/>
</p>
<p align="center">
Orchestration layer for Rig-based agents

## Overview
Mosaic is a Rust-native orchestration layer that enables Rig agents to collaborate and coordinate dynamically across complex tasks.

## Features
* 🤖 Deploy intelligent agents using Rig’s runtime and orchestration tools
* 🔗 Interaction with APIs, databases, and vector stores
* 🧩 Define multiple agents that work together dynamically
* ⚡ Optimized Rust architecture for low-latency execution

Mosaic works plug'n'play with the [$arc rig framework](https://github.com/0xPlaygrounds/rig) allowing Agents to interact with the Solana blockchain.

## Requirements
* Rust `1.75+` [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
* `cargo`
* `rig` (Optional, for advanced AI agent capabilities)
* Vector Database (Optional, for knowledge-based agents)

### Optional Dependencies
Mosaic supports additional tools for enhanced functionality:
* WebSockets / Message Queues - For distributed agent communication
* Graph Databases – If agents need structured relationship modeling (e.g.,[Neo4j](https://neo4j.com/))
* LLM APIs – If using external AI models like OpenAI, Anthropic, or Hugging Face

## Quickstart
This guide walks you through installing the library and deploying your first agents.

### Installation

From your project folder:
```
cargo add Mosaic
```
For Rig integration:
```
cargo add rig
```

### Define an Agent

Each agent has a role and a behavior. Here’s how to create one:
```rust
use mosaic::{Agent, Orchestrator};

fn main() {
    let agent = Agent::new("Researcher", |query| {
        format!("Researching: {}", query)
    });

    println!("Agent response: {}", agent.run("AI in Rust"));
}
```
*This creates an agent named `Researcher`, which processes queries and returns insights.*

### Orchestrate Multiple Agents

Mosaic enables agents to collaborate within an Orchestrator:
```rust
fn main() {
    let agent1 = Agent::new("Researcher", |query| {
        format!("Finding data on: {}", query)
    });

    let agent2 = Agent::new("Summarizer", |text| {
        format!("Summary: {}", text)
    });

    let mut orchestrator = Orchestrator::new();
    orchestrator.add_agent(agent1);
    orchestrator.add_agent(agent2);

    let result = orchestrator.process("Latest AI advancements");
    println!("Final Output: {}", result);
}
```
*Now, the `Researcher` and `Summarizer` agents work together to handle tasks!*

### Scale with Multiple Agents

Expand your ecosystem with domain-specific agents:
```rust
let coder = Agent::new("Code Assistant", |request| {
    format!("Generating Rust code for: {}", request)
});

let security = Agent::new("Security Auditor", |code| {
    format!("Analyzing security risks in: {}", code)
});
```
*This setup allows agents to specialize and collaborate on advanced tasks.*

### Using Rig to Extend Your Agent Ecosystem

Mosaic is natively compatible with Rig, allowing you to enhance your agents with LLMs

#### Add LLM Capabilities to an Agent
With Rig, you can add an AI agent that queries a language model dynamically:
```rust
use rig::llm::{LLM, OpenAI};
use mosaic::Agent;

fn main() {
    let ai_agent = Agent::new("AI Assistant", |query| {
        let llm = OpenAI::default();
        let response = llm.complete(query).unwrap();
        format!("AI Response: {}", response)
    });

    println!("{}", ai_agent.run("Explain Rust's ownership model"));
}
```
*Now, the agent can process natural language queries using OpenAI or any LLM supported by Rig.*

#### Use Rig for Tool-Using Agents
Here’s how an agent might retrieve real-time data:
```rust
use rig::tool::{Tool, HttpTool};
use mosaic::Agent;

fn main() {
    let web_scraper = Agent::new("Web Researcher", |query| {
        let tool = HttpTool::new();
        let response = tool.get(&format!("https://api.example.com/search?q={}", query)).unwrap();
        format!("Found: {}", response)
    });

    println!("{}", web_scraper.run("latest AI advancements"));
}
```
*This allows agents to fetch and process live web data.*

#### Scaling Further: Distributed Agents with Rig

Rig enables distributed execution, so agents in Mosaic can operate across multiple nodes, sharing tasks:
* Run agents in parallel for faster task completion
* Connect agents to message queues for real-time collaboration
* Store and retrieve agent memory using vector databases

### ARC Integration
Mosaic is designed to work with the ARC Rig framework, enabling enhanced multi-agent coordination, execution, and scalability.

#### Getting started

Install the necessary dependencies:
```
cargo add arc-runtime
```

Register agents within ARC:
```rust
use arc_runtime::{ArcEngine, ArcAgent};
use mosaic::{Agent, Orchestrator};

fn main() {
    let agent = ArcAgent::new("Data Analyst", |query| {
        format!("Fetching insights for: {}", query)
    });

    let mut arc = ArcEngine::new();
    arc.register(agent);

    arc.run();
}
```


## Architecture 

Mosaic enables the orchestration of multiple AI agents, allowing them to work collaboratively on complex tasks. Below is a high-level overview of how agents interact within the system:

```less
+-----------------+
|   User Query    |
+-----------------+
        |
        v
+--------------------+
|    Orchestrator   |
+--------------------+
   |       |       |
   v       v       v
[Agent A] [Agent B] [Agent C]
   |       |       |
   v       v       v
[Database] [API]  [Vector Store]
```
*The orchestrator dynamically assigns tasks to agents based on their capabilities, retrieving necessary information from external sources when needed.*

### Extensibility 
* Developers can define custom agents and integrate them easily
* Future versions may support multi-node agent collaboration
* Agents can retain context using vector databases like LanceDB or Qdrant
