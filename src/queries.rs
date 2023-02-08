//! This is an automatically generated module through the `generate`
//! subcommand of `graphql_client_cli`. See more [here][graphql_client_cli]
//!
//! We don't rely on the derive macro provided by [`graphql`] because we
//! want to deserialize [`BigInt`] from a string denoting the number, whereas
//! the type's implementation of [`Deserialize`][`serde::Deserialize`] does not
//! support this.
//!
//! Additional tweaks for readability and ergonomy are implemented.
//!
//! [graphql_client_cli]: https://github.com/graphql-rust/graphql-client#alternative-workflow-using-the-cli
#![allow(clippy::all, warnings, dead_code)]
use std::result::Result;

use bigdecimal::num_traits::ToPrimitive;
use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use serde::Deserializer;

pub struct GetPools;
pub type Pool = GetPoolsPools;

pub const OPERATION_NAME: &str = "GetPools";
/// Query to get a batch of pools that have liquidity greater than 0
/// ordered by total transaction count in reverse order.
pub const QUERY: &str = "query GetPool($first: Int! $skip: Int!) {\n    pools(\n      skip: $skip\n      first: $first\n      orderBy: txCount\n      orderDirection: desc\n where: {liquidity_gt: 0}\n ) {\n      token0 {\n        symbol\n      }\n      token1 {\n        symbol\n      }\n      feeTier\n      token0Price\n      token1Price\n    }\n  }\n" ;
use serde::{Deserialize, Serialize};
#[allow(dead_code)]
type Boolean = bool;
#[allow(dead_code)]
type Float = f64;
#[allow(dead_code)]
pub(crate) type Int = i64;
#[allow(dead_code)]
type ID = String;
#[derive(Serialize, Debug)]
pub struct Variables {
    pub first: Int,
    pub skip: Int,
}
impl Variables {}
#[derive(Deserialize, Debug)]
pub struct ResponseData {
    pub pools: Vec<GetPoolsPools>,
}
#[derive(Deserialize, Debug)]
pub struct GetPoolsPools {
    pub token0: GetPoolsPoolsToken0,
    pub token1: GetPoolsPoolsToken1,
    #[serde(rename = "feeTier", deserialize_with = "deserialize_fee_tier")]
    pub fee_tier: f64,
    #[serde(rename = "token0Price")]
    pub token0_price: BigDecimal,
    #[serde(rename = "token1Price")]
    pub token1_price: BigDecimal,
}
#[derive(Deserialize, Debug)]
pub struct GetPoolsPoolsToken0 {
    pub symbol: String,
}
#[derive(Deserialize, Debug)]
pub struct GetPoolsPoolsToken1 {
    pub symbol: String,
}

fn deserialize_bigint<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
    D: Deserializer<'de>,
{
    let number = String::deserialize(deserializer)?;
    Ok(BigInt::parse_bytes(number.as_bytes(), 10).unwrap_or_default())
}

fn deserialize_fee_tier<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let tier = deserialize_bigint(deserializer)?
        .to_f64()
        .expect("the fee tier should be representable in f64");
    Ok(tier / 1_000_000_f64)
}

impl graphql_client::GraphQLQuery for GetPools {
    type Variables = Variables;
    type ResponseData = ResponseData;
    fn build_query(variables: Self::Variables) -> ::graphql_client::QueryBody<Self::Variables> {
        graphql_client::QueryBody {
            variables,
            query: QUERY,
            operation_name: OPERATION_NAME,
        }
    }
}
