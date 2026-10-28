#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env,
};

/// Test fixture bundling deal engine client, SAC tokens, and test actor accounts.
pub struct E2EFixture<'a> {
    pub env: &'a Env,
    pub admin: Address,
    pub creator: Address,
    pub treasury: Address,
    pub seller: Address,
    pub buyers: [Address; 3],
    pub token_client: token::Client<'a>,
    pub token_admin_client: StellarAssetClient<'a>,
    pub deal_client: BazaarDealEngineContractClient<'a>,
    pub token_address: Address,
    pub contract_id: Address,
}

impl<'a> E2EFixture<'a> {
    pub fn new(env: &'a Env) -> Self {
        env.mock_all_auths();

        let admin = Address::generate(env);
        let creator = Address::generate(env);
        let treasury = Address::generate(env);
        let seller = Address::generate(env);
        let buyers = [
            Address::generate(env),
            Address::generate(env),
            Address::generate(env),
        ];

        // Register SAC token
        let token_admin = Address::generate(env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin);
        let token_address = token_contract.address();
        let token_client = token::Client::new(env, &token_address);
        let token_admin_client = StellarAssetClient::new(env, &token_address);

        // Mint starting balances: 100 XLM = 1,000,000,000 stroops to each buyer
        for b in &buyers {
            token_admin_client.mint(b, &1_000_000_000);
        }

        // Register and initialize Bazaar Deal Engine
        let contract_id = env.register(BazaarDealEngineContract, ());
        let deal_client = BazaarDealEngineContractClient::new(env, &contract_id);
        deal_client.initialize(&admin);

        // Configure 1% protocol fee (100 bps) with dedicated treasury
        deal_client.set_protocol_fee(&admin, &100);
        deal_client.set_treasury(&admin, &treasury);

        Self {
            env,
            admin,
            creator,
            treasury,
            seller,
            buyers,
            token_client,
            token_admin_client,
            deal_client,
            token_address,
            contract_id,
        }
    }
}

#[test]
fn test_e2e_fixture_initialization() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    assert_eq!(fix.deal_client.get_protocol_fee_bps(), 100);
    assert_eq!(fix.deal_client.get_treasury(), Some(fix.treasury.clone()));
    assert_eq!(fix.token_client.balance(&fix.buyers[0]), 1_000_000_000);
}
