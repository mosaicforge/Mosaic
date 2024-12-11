//! This example demonstrates simple default integration with [`axum`].

use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    response::Html,
    routing::{get, on, MethodFilter},
    Extension, Router,
};
use clap::{Args, Parser};
use futures::stream::{BoxStream, StreamExt as _};
use juniper::{
    graphql_object, graphql_subscription, graphql_value, EmptyMutation, EmptySubscription, FieldError, GraphQLInterface, GraphQLObject, GraphQLScalar, RootNode
};
use juniper_axum::{graphiql, graphql, playground};
use kg_node::kg;
// use juniper_graphql_ws::ConnectionConfig;
use tokio::{net::TcpListener, time::interval};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// use tokio_stream::wrappers::IntervalStream;

#[derive(Clone)]
pub struct Query {
    kg_client: Arc<kg_node::kg::Client>,
}

#[graphql_object]
impl Query {
    async fn node(&self, id: String) -> Option<Node> {
        self.kg_client.find_node_by_id::<HashMap<String, serde_json::Value>>(&id)
            .await
            .expect("Failed to find node")
            .map(Node::from)
    }
}

#[derive(Clone, Debug, GraphQLScalar)]
#[graphql(with = node_data)]
pub struct Attributes(HashMap<String, serde_json::Value>);

mod node_data {
    use juniper::{InputValue, ParseScalarResult, ScalarToken, ScalarValue, Value};

    use super::*;

    fn serde_to_graphql<S: ScalarValue>(v: &serde_json::Value) -> Value<S> {
        match v {
            serde_json::Value::String(s) => Value::scalar(s.clone()),
            serde_json::Value::Number(n) => Value::scalar(n.as_i64().unwrap() as i32),
            serde_json::Value::Bool(b) => Value::scalar(*b),
            serde_json::Value::Array(a) => Value::List(a.iter().map(serde_to_graphql).collect()),
            _ => Value::null(),
        }
    }

    pub(super) fn to_output<S: ScalarValue>(v: &Attributes) -> Value<S> {
        Value::Object(v.0.iter().fold(
            juniper::Object::with_capacity(v.0.len()),
            |mut obj, (k, v)| {
                obj.add_field(k, serde_to_graphql(v));
                obj
            },
        ))
    }

    pub(super) fn from_input<S: ScalarValue>(v: &InputValue<S>) -> Result<Attributes, String> {
        // v.as_string_value()
        //     .map(|s| StringOrInt::String(s.into()))
        //     .or_else(|| v.as_int_value().map(StringOrInt::Int))
        //     .ok_or_else(|| format!("Expected `String` or `Int`, found: {v}"))
        unimplemented!()
    }

    pub(super) fn parse_token<S: ScalarValue>(t: ScalarToken<'_>) -> ParseScalarResult<S> {
        unimplemented!()
    }
}

#[derive(Clone, Debug, GraphQLObject)]
pub struct Node {
    id: String,
    types: Vec<String>,
    attributes: Attributes,
    // relations_to: Vec<Relation>,
    // relations_from: Vec<Relation>,
}

impl From<kg::mapping::Node<HashMap<String, serde_json::Value>>> for Node 
{
    fn from(node: kg::mapping::Node<HashMap<String, serde_json::Value>>) -> Self {
        Self {
            id: node.id().to_string(),
            types: node.types,
            attributes: Attributes(node.attributes.attributes),
        }
    }
}

pub struct Relation {
    id: String,
    type_id: String,
    attributes: Attributes,
    to: Node,
    from: Node,
}

type Schema = RootNode<'static, Query, EmptyMutation, EmptySubscription>;

async fn homepage() -> Html<&'static str> {
    "<html><h1>juniper_axum/simple example</h1>\
           <div>visit <a href=\"/graphiql\">GraphiQL</a></div>\
           <div>visit <a href=\"/playground\">GraphQL Playground</a></div>\
    </html>"
        .into()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_log_level();
    init_tracing();

    let args = AppArgs::parse();

    let kg_client = kg_node::kg::Client::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;

    let schema = Schema::new(
        Query {
            kg_client: Arc::new(kg_client),
        },
        EmptyMutation::new(),
        EmptySubscription::new(),
    );

    let app = Router::new()
        .route(
            "/graphql",
            on(
                MethodFilter::GET.or(MethodFilter::POST),
                graphql::<Arc<Schema>>,
            ),
        )
        // .route(
        //     "/subscriptions",
        //     get(ws::<Arc<Schema>>(ConnectionConfig::new(()))),
        // )
        .route("/graphiql", get(graphiql("/graphql", "/subscriptions")))
        .route("/playground", get(playground("/graphql", "/subscriptions")))
        .route("/", get(homepage))
        .layer(Extension(Arc::new(schema)));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to listen on {addr}: {e}"));
    tracing::info!("listening on {addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("failed to run `axum::serve`: {e}"));

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,
}

#[derive(Debug, Args)]
struct Neo4jArgs {
    /// Neo4j database host
    #[arg(long)]
    neo4j_uri: String,

    /// Neo4j database user name
    #[arg(long)]
    neo4j_user: String,

    /// Neo4j database user password
    #[arg(long)]
    neo4j_pass: String,
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stdout=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn set_log_level() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}
