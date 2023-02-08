//! Collect uniswap pool data into a graph
use std::collections::HashMap;

use bigdecimal::BigDecimal;
use petgraph::algo::all_simple_paths;
use petgraph::prelude::{DiGraph, NodeIndex};

use crate::queries::Pool;

/// Represent the swap of a token
#[derive(Debug, Clone, Default)]
pub struct Swap {
    /// The symbol of the token to be exchanged
    token: String,
    /// The exchange rate
    rate: BigDecimal,
}

/// Represent the swap of a token
#[derive(Debug, Clone, Default)]
pub struct SwapPair {
    swap0: Swap,
    swap1: Swap,
}

impl From<Pool> for SwapPair {
    fn from(pool: Pool) -> Self {
        let token = pool.token0.symbol;
        let net = 1_f64 - pool.fee_tier;
        let rate = net / pool.token0_price;
        let swap0 = Swap { token, rate };
        let token = pool.token1.symbol;
        let rate = net / pool.token1_price;
        let swap1 = Swap { token, rate };
        Self { swap0, swap1 }
    }
}

/// The graph of pools
#[derive(Debug, Clone, Default)]
pub struct PoolGraph {
    inner: DiGraph<String, Swap>,
    symbols: HashMap<String, NodeIndex<u32>>,
}

impl From<Vec<Pool>> for PoolGraph {
    fn from(pools: Vec<Pool>) -> Self {
        let mut graph = Self::default();
        for pair in pools.into_iter().map(SwapPair::from) {
            if !graph.symbols.contains_key(&pair.swap0.token) {
                let ix = graph.inner.add_node(pair.swap0.token.clone());
                graph.symbols.insert(pair.swap0.token.clone(), ix);
            }
            if !graph.symbols.contains_key(&pair.swap1.token) {
                let ix = graph.inner.add_node(pair.swap1.token.clone());
                graph.symbols.insert(pair.swap1.token.clone(), ix);
            }
            let (token0, token1) = (
                graph.symbols[&pair.swap0.token],
                graph.symbols[&pair.swap1.token],
            );
            if let Some(edge) = graph.inner.find_edge(token0, token1) {
                let swap = &mut graph.inner[edge];
                if swap.rate < pair.swap0.rate {
                    swap.rate = pair.swap0.rate.clone();
                }
            } else {
                graph.inner.add_edge(token0, token1, pair.swap0.clone());
            }
            if let Some(edge) = graph.inner.find_edge(token1, token0) {
                let swap = &mut graph.inner[edge];
                if swap.rate < pair.swap1.rate {
                    swap.rate = pair.swap1.rate.clone();
                }
            } else {
                graph.inner.add_edge(token1, token0, pair.swap1.clone());
            }
        }
        graph
    }
}

impl PoolGraph {
    /// Default number of max intermediate nodes
    /// while finding the optimal exchange route
    const OPTIMAL_ROUTE_DEPTH: usize = 3;

    /// Get a sequence of supported tokens
    pub fn tokens(&self) -> Vec<&str> {
        self.symbols.keys().map(|t| t.as_str()).collect()
    }

    /// Calculate the total exchange rate of a route
    /// as the product of the exchange rate of each edge
    /// in the path
    fn route_exchange_rate(&self, nodes: &Vec<NodeIndex<u32>>) -> BigDecimal {
        nodes
            .iter()
            .take(nodes.len() - 1)
            .zip(nodes.iter().skip(1))
            .map(|(n0, n1)| {
                let edge_ix = self
                    .inner
                    .find_edge(*n0, *n1)
                    .expect("the nodes should exist");
                self.inner[edge_ix].rate.clone()
            })
            .reduce(|acc, e| acc * e)
            .unwrap_or_default()
    }

    fn route_symbols(&self, nodes: Vec<NodeIndex<u32>>) -> Vec<&str> {
        nodes
            .iter()
            .map(|n| {
                self.inner
                    .node_weight(*n)
                    .expect("this node should exist")
                    .as_str()
            })
            .collect()
    }

    /// Get all swap routes between two tokens sorted by max exchange rate.
    ///
    /// The effective route swap rate is returned along with each route.
    pub fn routes(
        &self,
        token0: impl AsRef<str>,
        token1: impl AsRef<str>,
        max_intermediate_nodes: usize,
    ) -> Vec<(Vec<&str>, BigDecimal)> {
        let mut routes: Vec<(Vec<NodeIndex<u32>>, BigDecimal)> = all_simple_paths(
            &self.inner,
            self.symbols[&token0.as_ref().to_string()],
            self.symbols[&token1.as_ref().to_string()],
            0,
            Some(max_intermediate_nodes),
        )
        .map(|path| {
            let score = self.route_exchange_rate(&path);
            (path, score)
        })
        .collect();
        routes.sort_unstable_by_key(|(_, score)| score.clone());
        routes.reverse();
        routes
            .into_iter()
            .map(|(path, score)| (self.route_symbols(path), score))
            .collect()
    }

    /// Get the optimal swap route, if any, between two tokens along with the
    /// effective exchange rate.
    pub fn optiomal_route(
        &self,
        token0: impl AsRef<str>,
        token1: impl AsRef<str>,
    ) -> Option<(Vec<&str>, BigDecimal)> {
        self.routes(token0, token1, Self::OPTIMAL_ROUTE_DEPTH)
            .get(0)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::UniswapClient;

    async fn construct_graph() -> PoolGraph {
        let client = UniswapClient::default();
        let n_pools = 1000;
        let batch_size = 100;
        println!("==> Constructing graph");
        PoolGraph::from(
            client
                .get_all_pools(Some(n_pools), Some(batch_size))
                .await
                .unwrap(),
        )
    }

    #[tokio::test]
    async fn graph_optimal_route() {
        let graph = construct_graph().await;
        println!("==> Calculating optimal route");
        let route = graph.optiomal_route("WETH", "LINK");
        println!("Optimal route {:#?}", route);
    }

    #[tokio::test]
    async fn graph_get_routes() {
        let graph = construct_graph().await;
        println!("==> Calculating routes");
        let routes = graph.routes("WETH", "LINK", 3);
        println!("Routes {:#?}", &routes[..]);
    }
}
