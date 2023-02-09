use parking_lot::RwLock;
use std::sync::Arc;

use async_graphql::{Context, Object};
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{self, IntoResponse},
    routing::get,
    Router, Server,
};

use uniswap_routing::client::UniswapClient;
use uniswap_routing::graph::PoolGraph;

pub struct QueryRoot;

pub type QueryGraph = Arc<RwLock<PoolGraph>>;
pub type UniswapRoutingSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[Object]
impl QueryRoot {
    /// Get a list of supported tokens for swaps
    async fn tokens(&self, ctx: &Context<'_>) -> anyhow::Result<Vec<String>> {
        let graph = ctx.data_unchecked::<QueryGraph>();
        Ok(graph
            .read()
            .tokens()
            .into_iter()
            .map(|n| n.to_string())
            .collect())
    }

    /// Get the optimal swap route for a pair of tokens
    async fn optimal_route(
        &self,
        ctx: &Context<'_>,
        token0: String,
        token1: String,
    ) -> anyhow::Result<Vec<String>> {
        let graph = ctx.data_unchecked::<QueryGraph>();
        let guard = graph.read();
        let (route, _) = guard
            .optimal_route(token0.as_str(), token1.as_str())
            .unwrap_or_default();
        Ok(route.into_iter().map(|n| n.to_string()).collect())
    }
}

async fn graphql_handler(
    schema: Extension<UniswapRoutingSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/").finish())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let uniswap_client = UniswapClient::default();
    let pools = uniswap_client.get_all_pools(Some(1000), Some(100)).await?;
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(Arc::new(RwLock::new(PoolGraph::from(pools))))
        .finish();

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema));

    println!("GraphiQL IDE: http://localhost:8000");

    Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
