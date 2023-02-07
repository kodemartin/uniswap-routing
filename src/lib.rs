//use graphql_client::{GraphQLQuery, Response};
use std::error::Error;

use graphql_client::{GraphQLQuery, Response};

mod get_pools;

use get_pools::GetPools;

const REQUEST_RETRIES: u8 = 3;
const UNISWAP_API: &str = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3";

pub async fn get_pools(variables: get_pools::Variables) -> Result<(), Box<dyn Error>> {
    let request_body = GetPools::build_query(variables);

    let client = reqwest::Client::new();
    let mut n_retries = REQUEST_RETRIES;
    while n_retries > 0 {
        println!("Attempts left #{n_retries}");
        let res = client.post(UNISWAP_API).json(&request_body).send().await?;
        let response_body: Response<get_pools::ResponseData> = res.json().await?;
        println!("{response_body:#?}");
        if response_body.data.is_none() {
            n_retries -= 1;
        } else {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let variables = get_pools::Variables {
            from_token: "LINK".into(),
            to_token: "USDC".into(),
            skip: 0,
            first: 10,
        };
        get_pools(variables).await.unwrap();
    }
}
