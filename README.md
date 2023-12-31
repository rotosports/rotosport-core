# Rotosports Core

[![codecov](https://codecov.io/gh/rotosports-fi/rotosports-core/branch/main/graph/badge.svg?token=ROOLZTGZMM)](https://codecov.io/gh/rotosports-fi/rotosports-core)

Multi pool type automated market-maker (AMM) protocol powered by smart contracts on the [Terra](https://terra.money) blockchain.

## Contracts diagram

![contract diagram](./assets/sc_diagram.png "Contracts Diagram")

## General Contracts

| Name                                                       | Description                                  |
| ---------------------------------------------------------- | -------------------------------------------- |
| [`factory`](contracts/factory)                             | Pool creation factory                        |
| [`pair`](contracts/pair)                                   | Pair with x*y=k curve                        |
| [`pair_stable`](contracts/pair_stable)                     | Pair with stableswap invariant curve         |
| [`pair_stable_bluna`](contracts/pair_stable_bluna)         | Pair with stableswap invariant curve handling bLUNA rewards for LPs |
| [`token`](contracts/token)                                 | CW20 (ERC20 equivalent) token implementation |
| [`router`](contracts/router)                               | Multi-hop trade router                       |
| [`oracle`](contracts/periphery/oracle)                     | TWAP oracles for x*y=k pool types            |
| [`whitelist`](contracts/whitelist)                         | CW1 whitelist contract                       |

## Tokenomics Contracts

Tokenomics related smart contracts are hosted on ../contracts/tokenomics.

| Name                                                       | Description                                      |
| ---------------------------------------------------------- | ------------------------------------------------ |
| [`generator`](contracts/tokenomics/generator)                                   | Rewards generator for liquidity providers        |
| [`generator_proxy_to_mirror`](contracts/tokenomics/generator_proxy_to_mirror)   | Rewards generator proxy for liquidity providers  |
| [`maker`](contracts/tokenomics/maker)                                           | Fee collector and swapper                        |
| [`staking`](contracts/tokenomics/staking)                                       | xROTO staking contract                          |
| [`vesting`](contracts/tokenomics/vesting)                                       | ROTO distributor for generator rewards          |
| [`xroto_token`](contracts/tokenomics/xroto_token)                             | xROTO token contract                            |

## Building Contracts

You will need Rust 1.64.0+ with wasm32-unknown-unknown target installed.

### You can compile each contract:
Go to contract directory and run 
    
```
cargo wasm
cp ../../target/wasm32-unknown-unknown/release/rotosports_token.wasm .
ls -l rotosports_token.wasm
sha256sum rotosports_token.wasm
```

### You can run tests for all contracts
Run the following from the repository root

```
cargo test
```

### For a production-ready (compressed) build:
Run the following from the repository root

```
./scripts/build_release.sh
```

The optimized contracts are generated in the artifacts/ directory.

## Deployment

You can find versions and commits for actually deployed contracts [here](https://github.com/rotosports-fi/rotosports-changelog).

## Docs

Docs can be generated using `cargo doc --no-deps`

## Bug Bounty

The contracts in this repo are included in a [bug bounty program](https://www.immunefi.com/bounty/rotosports).
