use thales_runtime as thales;
use thales_runtime::{
    constants::{currency::*},
    AccountId, EvmAddress, GenesisConfig, Signature, WASM_BINARY,
};
use hex_literal::hex;
use jsonrpc_core::serde_json::Map;
use pallet_evm::GenesisAccount;
use sc_service::{config::TelemetryEndpoints, ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{
    crypto::{UncheckedInto},
    sr25519, Pair, Public, H160, U256,
};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
};
use std::{str::FromStr};

// The URL for the telemetry server.
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "thales";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                vec![H160::from_str("1DfeF571E6E4418a3E170889DE0e4E784FeA1E4a").unwrap()],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(properties()),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("Development wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                vec![H160::from_str("1DfeF571E6E4418a3E170889DE0e4E784FeA1E4a").unwrap()],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                ],
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(properties()),
        // Extensions
        None,
    ))
}

pub fn thales_staging_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or("thales development wasm not available")?;
    let boot_nodes = vec![];

    Ok(ChainSpec::from_genesis(
        "Thaleschain Staging Testnet",
        "thales_staging_testnet",
        ChainType::Live,
        move || thales_staging_config_genesis(wasm_binary),
        boot_nodes,
        Some(
            TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
                .expect("Thaleschain Staging telemetry url is valid; qed"),
        ),
        Some(DEFAULT_PROTOCOL_ID),
        Some(properties()),
        Default::default(),
    ))
}

fn thales_staging_config_genesis(wasm_binary: &[u8]) -> GenesisConfig {
    testnet_genesis(
        wasm_binary,
        vec![
            (
                hex!["282096d77ffea749cacb77332d3bc97a5faa2070dd103e8d0faa4f68e8f12922"]
                    .unchecked_into(),
                hex!["80396502034665c5c491a2a31592fefadb613b6498bcf8d6a9245949416b92a5"]
                    .unchecked_into(),
            ),
            (
                hex!["eecc70b5635b6508c38733035b3d78cd52798a2e69cad299b1626e4f789d3330"]
                    .unchecked_into(),
                hex!["be5039c105a74be1f14621fbe22bbe61e5b365816cc5cc58421a3e08c7b8c968"]
                    .unchecked_into(),
            ),
            (
                hex!["7e5df99677c14dc4019b9c7a20dcc4f9895ceb855481320401985e5cfd481525"]
                    .unchecked_into(),
                hex!["1229bedae699d949be52d3029dd8d7fd61db8fbefbd0b87b06404ff6140487ea"]
                    .unchecked_into(),
            ),
        ],
        // 5CCh84seSNarfXKJ8oVcaL4PFgq88REujb8pBeJ6LyedDx74
        hex!["06177d9d1a4df0153c25216352265654fe4f0bc1280a6ab1d3b8c542a08d6174"].into(),
        vec![
            // 5CCh84seSNarfXKJ8oVcaL4PFgq88REujb8pBeJ6LyedDx74
            hex!["06177d9d1a4df0153c25216352265654fe4f0bc1280a6ab1d3b8c542a08d6174"].into(),
        ],
        vec![H160::from_str("50129bc92EaC7f390a07e9AEc649b67B47A2B3F0").unwrap()],
        vec![],
        vec![],
        true,
    )
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    evm_accounts: Vec<EvmAddress>,
    council: Vec<AccountId>,
    technical_committee: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: thales::SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_balances: thales::BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 10_000_000_000 * DOLLARS))
                .collect(),
        },
        pallet_aura: thales::AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        pallet_grandpa: thales::GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        pallet_sudo: thales::SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        pallet_evm: thales::EVMConfig {
            accounts: evm_accounts
                .iter()
                .cloned()
                .map(|k| {
                    let account = GenesisAccount {
                        nonce: U256::from(0),
                        balance: U256::from(1_000_000_000 * DOLLARS),
                        storage: Default::default(),
                        code: vec![],
                    };
                    (k, account)
                })
                .collect(),
            // accounts: BTreeMap::new(),
        },
        pallet_ethereum: Default::default(),
        pallet_collective_Instance1: thales::CouncilConfig {
            members: council,
            phantom: Default::default(),
        },
        pallet_collective_Instance2: Default::default(),
        pallet_membership_Instance1: thales::TechnicalMembershipConfig {
            members: technical_committee,
            phantom: Default::default(),
        },
        pallet_elections_phragmen: Default::default(),
    }
}

fn properties() -> Properties {
    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "THALA".into());
    properties.insert("tokenDecimals".into(), 18.into());
    return properties;
}
