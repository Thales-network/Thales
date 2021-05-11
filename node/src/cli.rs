// Copyright 2019-2021 Thales Inc.
// This file is part of Thales.

// Thales is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Thales is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Thales.  If not, see <http://www.gnu.org/licenses/>.

//! Thales CLI Library. Built with structopt
//!
//! This module defines the Thales node's Command Line Interface (CLI)
//! It is built using structopt and inherits behavior from Substrate's sc_cli crate.

use crate::chain_spec;
use sp_core::H160;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

/// Sub-commands supported by the collator.
#[derive(Debug, StructOpt)]
pub enum Subcommand {
	/// Export the genesis state of the parachain.
	#[structopt(name = "export-genesis-state")]
	ExportGenesisState(ExportGenesisStateCommand),

	/// Export the genesis wasm of the parachain.
	#[structopt(name = "export-genesis-wasm")]
	ExportGenesisWasm(ExportGenesisWasmCommand),

	/// Build a chain specification.
	BuildSpec(BuildSpecCommand),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Remove the whole chain.
	PurgeChain(cumulus_client_cli::PurgeChainCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// The custom benchmark subcommmand benchmarking runtime pallets.
	#[structopt(name = "benchmark", about = "Benchmark runtime pallets.")]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),
}

#[derive(Debug, StructOpt)]
pub struct BuildSpecCommand {
	#[structopt(flatten)]
	pub base: sc_cli::BuildSpecCmd,

	/// Number of accounts to be funded in the genesis
	/// Warning: This flag implies a development spec and overrides any explicitly supplied spec
	#[structopt(long, conflicts_with = "chain")]
	pub accounts: Option<u32>,

	/// Mnemonic from which we can derive funded accounts in the genesis
	/// Warning: This flag implies a development spec and overrides any explicitly supplied spec
	#[structopt(long, conflicts_with = "chain")]
	pub mnemonic: Option<String>,
}

/// Command for exporting the genesis state of the parachain
#[derive(Debug, StructOpt)]
pub struct ExportGenesisStateCommand {
	/// Output file name or stdout if unspecified.
	#[structopt(parse(from_os_str))]
	pub output: Option<PathBuf>,

	/// Id of the parachain this state is for.
	#[structopt(long, default_value = "1000")]
	pub parachain_id: u32,

	/// Write output in binary. Default is to write in hex.
	#[structopt(short, long)]
	pub raw: bool,

	/// The name of the chain for that the genesis state should be exported.
	#[structopt(long)]
	pub chain: Option<String>,
}

/// Command for exporting the genesis wasm file.
#[derive(Debug, StructOpt)]
pub struct ExportGenesisWasmCommand {
	/// Output file name or stdout if unspecified.
	#[structopt(parse(from_os_str))]
	pub output: Option<PathBuf>,

	/// Write output in binary. Default is to write in hex.
	#[structopt(short, long)]
	pub raw: bool,

	/// The name of the chain for that the genesis wasm file should be exported.
	#[structopt(long)]
	pub chain: Option<String>,
}

#[derive(Debug, StructOpt)]
pub struct RunCmd {
	#[structopt(flatten)]
	pub base: sc_cli::RunCmd,

	/// Id of the parachain this collator collates for.
	#[structopt(long)]
	pub parachain_id: Option<u32>,

	/// Enable the development service to run without a backing relay chain
	#[structopt(long)]
	pub dev_service: bool,

	/// When blocks should be sealed in the dev service.
	///
	/// Options are "instant", "manual", or timer interval in milliseconds
	#[structopt(long, default_value = "instant")]
	pub sealing: Sealing,

	/// Public identity for participating in staking and receiving rewards
	#[structopt(long, parse(try_from_str = parse_h160))]
	pub author_id: Option<H160>,

	/// Enable EVM tracing module on a non-authority node.
	#[structopt(
		long,
		conflicts_with = "collator",
		conflicts_with = "validator",
		require_delimiter = true
	)]
	pub ethapi: Vec<EthApi>,

	/// Number of concurrent tracing tasks. Meant to be shared by both "debug" and "trace" modules.
	#[structopt(long, default_value = "10")]
	pub ethapi_max_permits: u32,

	/// Maximum number of trace entries a single request of `trace_filter` is allowed to return.
	/// A request asking for more or an unbounded one going over this limit will both return an
	/// error.
	#[structopt(long, default_value = "500")]
	pub ethapi_trace_max_count: u32,

	/// Duration (in seconds) after which the cache of `trace_filter` for a given block will be
	/// discarded.
	#[structopt(long, default_value = "300")]
	pub ethapi_trace_cache_duration: u64,
}

fn parse_h160(input: &str) -> Result<H160, String> {
	input
		.parse::<H160>()
		.map_err(|_| "Failed to parse H160".to_string())
}

impl std::ops::Deref for RunCmd {
	type Target = sc_cli::RunCmd;

	fn deref(&self) -> &Self::Target {
		&self.base
	}
}

#[derive(Debug, StructOpt)]
#[structopt(settings = &[
	structopt::clap::AppSettings::GlobalVersion,
	structopt::clap::AppSettings::ArgsNegateSubcommands,
	structopt::clap::AppSettings::SubcommandsNegateReqs,
])]
pub struct Cli {
	#[structopt(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[structopt(flatten)]
	pub run: RunCmd,

	/// Run node as collator.
	///
	/// Note that this is the same as running with `--validator`.
	#[structopt(long, conflicts_with = "validator")]
	pub collator: bool,

	/// Relaychain arguments
	#[structopt(raw = true)]
	pub relaychain_args: Vec<String>,
}

#[derive(Debug)]
pub struct RelayChainCli {
	/// The actual relay chain cli object.
	pub base: polkadot_cli::RunCmd,

	/// Optional chain id that should be passed to the relay chain.
	pub chain_id: Option<String>,

	/// The base path that should be used by the relay chain.
	pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain `Configuration`.
	pub fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let extension = chain_spec::Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = para_config
			.base_path
			.as_ref()
			.map(|x| x.path().join("polkadot"));
		Self {
			base_path,
			chain_id,
			base: polkadot_cli::RunCmd::from_iter(relay_chain_args),
		}
	}
}

/// Block authoring scheme to be used by the dev service.
#[derive(Debug)]
pub enum Sealing {
	/// Author a block immediately upon receiving a transaction into the transaction pool
	Instant,
	/// Author a block upon receiving an RPC command
	Manual,
	/// Author blocks at a regular interval specified in milliseconds
	Interval(u64),
}

impl FromStr for Sealing {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"instant" => Self::Instant,
			"manual" => Self::Manual,
			s => {
				let millis =
					u64::from_str_radix(s, 10).map_err(|_| "couldn't decode sealing param")?;
				Self::Interval(millis)
			}
		})
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum EthApi {
	Txpool,
	Debug,
	Trace,
}

impl FromStr for EthApi {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"txpool" => Self::Txpool,
			"debug" => Self::Debug,
			"trace" => Self::Trace,
			_ => {
				return Err(format!(
					"`{}` is not recognized as a supported Ethereum Api",
					s
				))
			}
		})
	}
}
