// This file is a part of Duality
//
// Copyright (c) 2022 Duality Blockchain Solutions LLC
// Copyright (c) 2017-2021 Parity Technologies (UK) Ltd.
//
// SPDX-License-Identifier: GPL-3.0-or-later
//

use sc_service::ChainType;
use sp_core::{ed25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

use runtime_sparrow::{*, constants::currency::*};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

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

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
	seed: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<ed25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<ed25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn development_config_genesis(wasm_binary: &[u8]) -> GenesisConfig {
	testnet_genesis(
		wasm_binary,
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice")],
		// Pre-funded accounts
		vec![
			get_account_id_from_seed::<ed25519::Public>("Alice"),
			get_account_id_from_seed::<ed25519::Public>("Bob"),
			get_account_id_from_seed::<ed25519::Public>("Alice//stash"),
			get_account_id_from_seed::<ed25519::Public>("Bob//stash"),
		],
		true,
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || development_config_genesis(wasm_binary),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}


fn testnet_config_genesis(wasm_binary: &[u8]) -> GenesisConfig {
	testnet_genesis(
		wasm_binary,
		// Initial PoA authorities
		vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
		// Pre-funded accounts
		vec![
			get_account_id_from_seed::<ed25519::Public>("Alice"),
			get_account_id_from_seed::<ed25519::Public>("Bob"),
			get_account_id_from_seed::<ed25519::Public>("Charlie"),
			get_account_id_from_seed::<ed25519::Public>("Dave"),
			get_account_id_from_seed::<ed25519::Public>("Eve"),
			get_account_id_from_seed::<ed25519::Public>("Ferdie"),
			get_account_id_from_seed::<ed25519::Public>("Alice//stash"),
			get_account_id_from_seed::<ed25519::Public>("Bob//stash"),
			get_account_id_from_seed::<ed25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<ed25519::Public>("Dave//stash"),
			get_account_id_from_seed::<ed25519::Public>("Eve//stash"),
			get_account_id_from_seed::<ed25519::Public>("Ferdie//stash"),
		],
		true,
	)
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || testnet_config_genesis(wasm_binary),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys {
		babe,
		grandpa,
		im_online,
		authority_discovery,
	}
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	const ENDOWMENT: u128 = 1_000_000 * COIN;
    const STASH: u128 = 100 * COIN;
	GenesisConfig {
		system: SystemConfig { code: wasm_binary.to_vec() },
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
		indices: IndicesConfig { indices: vec![] },
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone()
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			force_era: Forcing::ForceNone,
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		elections_phragmen: Default::default(),
		democracy: Default::default(),
		council: CouncilConfig { members: vec![], phantom: Default::default() },
		technical_committee: TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		},
		technical_membership: Default::default(),
		babe: BabeConfig {
			authorities: Default::default(),
			epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
		},
		grandpa: Default::default(),
		im_online: Default::default(),
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		vesting: VestingConfig { vesting: vec![] },
		treasury: Default::default(),
	}
}
