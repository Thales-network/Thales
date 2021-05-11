# Tools

_NB: this folder is yarn only_

## Thales-Launch

_Thales-launch_ (https://github.com/Thales-network/polkadot-launch/tree/thales-launch) is a fork of polkadot-launch adapted for Thales's needs for testing. Eventually, polkadot-launch should catch up and remove the need for a custom implementation.

_Polkadot-launch_ (https://github.com/paritytech/polkadot-launch) allows to start a local network of polkadot relaychain and parachain nodes with the desired configuration.

For this setup, you want to have thales and polkadot cloned in the same repo:
- repo
    - thales
    - polkadot

### Build Parachain

In the thales repo, checkout to the desired commit of thales and then run:

```
cargo build --release
./target/release/thales --version
```

### Build Relaychain

First, in the thales repo, look in the cargo.lock file to get the sha of the commit of the used polkadot version (ctrl+f `https://github.com/paritytech/polkadot`), or run any of the relay related scripts to see that sha logged.

Then, in the polkadot repo, cloned in the same repo as the thales repo, run:

```
git checkout <commit sha>
cargo build --release --features=real-overseer
```

### Launch Script

Run `yarn run build-thales-launch` to install the correct dependency
    - Installs Thales thales-launch branch

Run `yarn run thales-launch` to start a network with `config_thales.json`
    - Installs Thales thales-launch branch
    - Starts a local network with `config_thales.json`

Run `yarn run thales-test`, if you want to run a simple test sending transactions to different addresses:
    - Installs Thales thales-launch branch
    - Starts a local network with `config_thales.json`
    - Runs a simple test sending transactions and testing propagation

### Launch Custom Scripts

Before you run a custom script, run `yarn run build-thales-launch` to install the correct dependency

If you want to run the staking test, run `ts-node test-staking.ts`

If you want to write your own custom test, use the start function from `polkadot-launch` :

`import { start } from "polkadot-launch";`

And then you can call it with the desired test-config this way:

`await start("config_thales_staking.json");`

### Change Config

Change the path in the config_thales.json file to use polkadot in a different location.

### Understand The Config

Let's look at the staking test-config:

```
{
  "relaychain": { // This field corresponds to the relaychain nodes config
    "bin": "../../polkadot/target/release/polkadot", // The executable for the relay chain
    "chain": "rococo-local", // chain param for relay-chain
    "nodes": [ // # of nodes needs to be >= # of collator nodes
      {
        "name": "alice",
        "wsPort": 36944,
        "port": 36444
      },
      {
        "name": "bob",
        "wsPort": 36955,
        "port": 36555
      },
      {
        "name": "charlie",
        "wsPort": 36956,
        "port": 36556
      },
      {
        "name": "dave",
        "wsPort": 36957,
        "port": 36557
      }
    ]
  },
  "parachains": [ // This field corresponds to the parachain nodes config
    {
      "bin": "../target/release/thales", // parachain executable
      "id": "1000", // Parachain id, use the same id if collators are of the same parachain
      "rpcPort": 36846,
      "wsPort": 36946, // Don't forget to increment the ports
      "port": 36335,
      "balance": "1000", // Balance of relaychain tokens used by the parachain to register
      "chain": "staking-test-spec.json", // custom specs for the parachain
      "flags": [
        "--no-telemetry",
        "--no-prometheus",
        "--author-id=6be02d1d3665660d22ff9624b7be0551ee1ac91b",
        "--", // before this are the collator flags, after are the relaychain related flags
        "--execution=wasm"
      ]
    },
    {
      "bin": "../target/release/thales",
      "id": "1000", // this node is of the same parachain
      "rpcPort": 36847,
      "wsPort": 36947,
      "port": 36336,
      "balance": "1000",
      "chain": "staking-test-spec.json",
      "flags": [
        "--no-telemetry",
        "--no-prometheus",
        "--author-id=C0F0f4ab324C46e55D02D0033343B4Be8A55532d",
        "--",
        "--execution=wasm"
      ]
    },
    {
      "bin": "../target/release/thales",
      "id": "1000",
      "rpcPort": 36848,
      "wsPort": 36948,
      "port": 36337,
      "balance": "1000",
      "chain": "staking-test-spec.json",
      "flags": [
        "--no-telemetry",
        "--no-prometheus",
        "--author-id=Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB",
        "--",
        "--execution=wasm"
      ]
    }
  ],
  "simpleParachains": [], // This is used by paritytech to test "simple" (dev) parachains
  "hrmpChannels": [], // This is used to setup hrmp channels from the start
  // it would look like this:
  // {
  //		"sender": 200,
  // 		"recipient": 300,
  //		"maxCapacity": 8,
  //   		"maxMessageSize": 512
  // }
  "types": { // This is were relaychain types are added. We are adding this due to discrepancy
  // between polkadot repo and published polkadot rococo
    "Address": "MultiAddress",
    "LookupSource": "MultiAddress"
  }
}
```

### Generate Test Specs

To generate the specs, run :
`./target/release/thales build-spec --chain local >new-specs.json`
