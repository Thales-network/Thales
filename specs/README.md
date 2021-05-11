# Embedded Spec Files

This directory contains chain specs for well-known public networks.

## Context

The Thales node is designed to support multiple networks including Thales Alpha, ThalseKusama
(Kusama) and Thales (Polkadot). Some of these networks are already live and others are planned.

In order to support multiple networks with the same binary, Thales relies on a chain specification
to know which network to sync. Rather than require node operators to obtain spec files separately,
it is convenient to "bake" specs for popular networks into the node.

## Which specs will come pre-baked?

- Thales Stage V6 - internal
- Thales Alpha V6 - live
- ThalesRococo - Potential future deployment to Rococo
- ThalesKusama - Future Kusama Deployment
- Thales - Future Polkadot deployment

## Relay chain specs

Because Thales networks are parachains, each network instance requires both a parachain and a
relay chain spec. For popular relay chains like kusama and polkadot, we rely on the specs being
already included with Polkadot. For smaller relay chains, like the one that exists solely to support
thalesbase alpha, we also bake the relay spec into the thalesbase binary.
