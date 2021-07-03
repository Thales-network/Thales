# Thales Network

**An Ethereum compatible [Parachain](https://polkadot.network/technology/) built with [Substrate](https://substrate.dev).**

👉 _Discover the Thales project at [thales.network](https://thales.network)._<br>
👉 _Learn to [use the Thales network](https://docs.thales.network/) with our technical docs._<br>
👉 _Reference our [crate-level docs (rustdocs)](https://Thales-network.github.io/thales) to contribute._

## Performance

Thales network focus on performance. Thales network block interval is about 1 second, and TPS will reach 1000+ in 2021 Q3, and will believe TPS will reach 5000+ in 2022 Q2.

## Low cost

Thales network will reduce cost for users in following way:
1. Every address (not include contract) which balance exceed min balance will have several fee Tx fee per day.;
2. Contract can pay gas fee for user;
3. Gas fee will be anchored with stable coin, and if payed with Thales token, you will has some discount;

## DPos

Thales network will use Dpos consensus.

Thales network have max 21 main node, 11 backup node. 90% token will give to main node, and 10% to backup node. For main node.

## Two main coin: Thales & StableCoin

In the latter version, Thales will with two coin, Thales & StableCoin.

They have different usage, stable coin can only pay gas fee, and thales can pay gas fee, community governance, and more.

## Oracle
Thales will have oracle in later version.

## Run an alphanet node with Docker

Docker images are published for every tagged release. Learn more with `thales --help`.

```bash
# Join the public testnet
docker run --network="host" Thales-network/thales:v0.9 --chain alphanet
```

## Run a local development node with Docker

Developers who are building dApps to run on thales, may want a lightweight node to work with
locally. You can quickly spin up a single node with no relay chain backing it using the development
service.

```bash
# Run a dev service node.
docker run --network="host" Thales-network/thales:v0.9 --dev
```

### Sealing options

The command above will start the node in instant seal mode. It creates a block when a transaction arrives, similar to Ganache's auto-mine. You can also choose to author blocks at a regular interval, or control authoring manually through the RPC.

```bash
# Author a block every 6 seconds.
docker run --network="host" Thales-network/thales:v0.9 --dev --sealing 3000

# Manually control the block authorship and finality
docker run --network="host" Thales-network/thales:v0.9 --dev --sealing manual
```

### Prefunded Development Addresses

Running Thales in development mode will pre-fund several well-known addresses that (mostly) contain the letters "th" in their names to remind you that they are for ethereum-compatible usage. These addresses are derived from
Substrate's canonical mnemonic: `bottom drive obey lake curtain smoke basket hold race lonely fit walk`

```
# Alith:
- Address:0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac
- PrivKey:0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133

# Baltathar:
- Address:0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0
- PrivKey:0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b

# Charleth:
- Address:0x798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc
- PrivKey:0x0b6e18cafb6ed99687ec547bd28139cafdd2bffe70e6b688025de6b445aa5c5b

# Dorothy:
- Address:0x773539d4Ac0e786233D90A233654ccEE26a613D9
- PrivKey:0x39539ab1876910bbf3a223d84a29e28f1cb4e2e456503e7e91ed39b2e7223d68

# Ethan:
- Address:0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB
- PrivKey:0x7dce9bc8babb68fec1409be38c8e1a52650206a7ed90ff956ae8a6d15eeaaef4

# Faith:
- Address:0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d
- PrivKey:0xb9d2ea9a615f3165812e8d44de0d24da9bbd164b65c4f0573e1ce2c8dbd9c8df

# Gerald:
- Address:0x7BF369283338E12C90514468aa3868A551AB2929
- PrivKey:0x96b8a38e12e1a31dee1eab2fffdf9d9990045f5b37e44d8cc27766ef294acf18
```

## Build the Thales Node

To build Thales, you will need a proper Substrate development environment. If you've never worked
with a Substrate-based blockchain before, you should probably try the [Setting Up a Thales Node](https://docs.thales.network/getting-started/local-node/setting-up-a-node/) docs first. If you
need a refresher setting up your Substrate environment, see [Substrate's Getting Started Guide](https://substrate.dev/docs/en/knowledgebase/getting-started/).

```bash
# Fetch the code
git clone https://github.com/Thales-network/thales
cd thales

# Build the node (The first build will be long (~30min))
cargo build --release
```

## Run tests

Thales has Rust unit tests as well as typescript integration tests. These tests are run in CI, and can also be run locally.

```bash
# Run the Rust unit tests
cargo test
```

```bash
# Install dependencies for integration tests
cd thales-types-bundle
npm i

cd ../tests
npm i

# Run integration tests
npm test
```

## Chain IDs

The Ethereum specification described a numeric Chain Id. The Thales mainnet Chain Id will be 624
because thales born in 624.BC.

Thales nodes support multiple public chains and testnets, with the following Chain Ids.

| Network Description                | Chain ID    |
| ---------------------------------- | ----------- |
| Local Parachain TestNet            | 546        |
| Local Development TestNet          | 547        |
| Reserved for other TestNets        | 548-560    |
| Thales (Polkadot)                  | 624        |
| ThalesKusama (Kusama)              | 623        |
| ThalesRococo (Rococo)              | 622        |
| Thales Alpha TestNet               | 621        |
| Reserved for other public networks | 610-620    |

## Runtime Architecture

The Thales Runtime is built using FRAME and consists of pallets from substrate, frontier, cumulus, and `pallets/`.

From substrate:

- _Utility_: Allows users to use derivative accounts, and batch calls
- _Balances_: Tracks GLMR token balances
- _Sudo_: Allows a privileged account to make arbitrary runtime changes - will be removed before
  launch
- _Timestamp_: On-Chain notion of time
- _Transaction Payment_: Transaction payment (fee) management
- _Randomness Collective Flip_: A (mock) onchain randomness beacon. Will be replaced by parachain
  randomness by mainnet.

From frontier:

- _EVM_: Encapsulates execution logic for an Ethereum Virtual Machine
- _Ethereum_: Ethereum-style data encoding and access for the EVM.

From cumulus:

- _ParachainUpgrade_: A helper to perform runtime upgrades on parachains
- _ParachainInfo_: A place to store parachain-relevant constants like parachain id

The following pallets are stored in `pallets/`. They are designed for Thales's specific requirements:

- _Ethereum Chain Id_: A place to store the chain id for each Thales network
- _Author Inherent_: Allows block authors to include their identity in a block via an inherent
- _Parachain Staking_: Minimal staking pallet that selects collators by total amount at stake

## Contribute

Thales is open source under the terms of the GPL3. We welcome contributions. You can explore our
crate-level documentation at https://thales-network.github.io/thales

### Code style

Thales is following the
[Substrate code style](https://github.com/paritytech/substrate/blob/master/docs/STYLE_GUIDE.md)  
We provide a [.editorconfig](.editorconfig) (_compatible with VSCode using RLS_)
