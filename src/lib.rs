use graphql_client::{GraphQLQuery, Response};
use tokio::task::JoinSet;

mod error;
mod get_pools;

use get_pools::{GetPools, Pool};

const UNISWAP_API: &str = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3";

#[derive(Debug, Clone, Default)]
pub struct UniswapClient(reqwest::Client);

impl UniswapClient {
    /// Number of retries for requests to uniswap api
    const N_RETRIES: u8 = 3;
    /// The default batch size of pools to fetch
    const POOL_BATCH_SIZE: u16 = 100;
    /// The max number of pools to fetch
    const N_POOLS: u16 = 2000;

    async fn get_pools(&self, variables: get_pools::Variables) -> error::Result<Option<Vec<Pool>>> {
        let request_body = GetPools::build_query(variables);

        let mut n_retries = Self::N_RETRIES;
        while n_retries > 0 {
            let res = self.0.post(UNISWAP_API).json(&request_body).send().await?;
            let response_body: Response<get_pools::ResponseData> = res.json().await?;
            println!("attempts left: {n_retries}");
            println!("response: {response_body:#?}");
            if response_body.data.is_none() {
                n_retries -= 1;
            } else {
                return Ok(response_body.data.map(|d| d.pools));
            }
        }
        Err(error::Error::GetPoolsMaxRetries)
    }

    /// Get all pools up to a user-defined number in batches of
    /// the given size.
    ///
    /// This is guaranteed to fetch the `n_pools` number of pools
    /// with the most transactions, although the vector is not ordered.
    pub async fn get_all_pools(
        &self,
        n_pools: Option<u16>,
        batch_size: Option<u16>,
    ) -> error::Result<Vec<Pool>> {
        let n_pools = n_pools.unwrap_or(Self::N_POOLS);
        let batch_size = batch_size.unwrap_or(Self::POOL_BATCH_SIZE);
        let mut count = u16::default();
        // Make the requests
        let mut requests = JoinSet::new();
        while count < n_pools {
            let step = get_pools::Variables {
                first: batch_size as get_pools::Int,
                skip: count as get_pools::Int,
            };
            let client = self.clone();
            requests.spawn(async move { client.get_pools(step).await });
            count += batch_size;
        }

        // Collect the results
        let mut pools = Vec::with_capacity(n_pools as usize);
        while let Some(response) = requests.join_next().await {
            if let Ok(Ok(Some(mut fetched_pools))) = response {
                pools.append(&mut fetched_pools);
            }
        }
        Ok(pools)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn client_get_pools() {
        let client = UniswapClient::default();
        let n_pools = 10;
        let variables = get_pools::Variables {
            skip: 0,
            first: n_pools,
        };
        let pools = client.get_pools(variables).await.unwrap().unwrap();
        assert_eq!(pools.len(), n_pools as usize);
    }

    #[tokio::test]
    async fn client_get_all_pools() {
        let client = UniswapClient::default();
        let n_pools = 100;
        let batch_size = 10;
        let pools = client
            .get_all_pools(Some(n_pools), Some(batch_size))
            .await
            .unwrap();
        assert_eq!(pools.len(), n_pools as usize);
    }
}
