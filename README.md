# `uniswap-routing`

A small library that collects swap-pool data from [uniswap][] into a directed
graph in memory.

The library exposes the following features:

* Fetch pool data in batches
* Expose methods of the pool graph that
  * Provide the list of supported tokens
  * Provide all possible routes between two token pairs, along with the
    respective effective exchange rate
  * Provide the optimal route between two token pairs

## Web interface

This functionality is used in a GraphQl server that can be run with

```
RUST_LOG=debug cargo run
```

The server constructs the pool graph at startup, and detaches a task to update
it every minute in the background.

## Limitations/Future work

* Pools with liquidity equal to 0 are not processed.  Nevertheless, it appears
  that they provide info about "dummy" pairs like "ETH" - "WETH".
* Use aliases for common tokens that have a wrapped equivalent (e.g. "ETH" - "WETH")
* Filter out outliers: It seems that there are pools (e.g. [MASK/ETH][mask])
  that present abnormal exchange rates. Obviously the have different scope, but
  at any case they should be filtered out as they tamper with the query results

[uniswap]: https://info.uniswap.org
[mask]: https://info.uniswap.org/pools#/pools/0x4ebc76bba018abc76b18afc61c7345ea0af0a037
