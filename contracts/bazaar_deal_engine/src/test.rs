#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env,
};

#[test]
fn test_demand_circle_lifecycle_and_settlement() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer1 = Address::generate(&env);
    let buyer2 = Address::generate(&env);
    let seller = Address::generate(&env);

    // Register token contract and mint balances
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = token::Client::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&buyer1, &100_000_000); // 10 XLM (7 decimals)
    token_admin_client.mint(&buyer2, &100_000_000);

    // Register Bazaar contract
    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    // 1. Create a Demand Circle
    // Target unit price: 10_000_000 (1 XLM), Min volume: 5, Max volume: 20, Duration: 86400s
    let circle_id = client.create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &5,
        &20,
        &86400,
    );
    assert_eq!(circle_id, 1);

    let initial_circle = client.get_circle(&circle_id);
    assert_eq!(initial_circle.current_volume, 0);
    assert_eq!(initial_circle.status, CircleStatus::Open);

    // 2. Buyer 1 commits 3 units (Cost = 3 * 1 = 3 XLM = 30_000_000)
    client.commit_demand(&buyer1, &circle_id, &3);
    assert_eq!(token_client.balance(&buyer1), 70_000_000);
    assert_eq!(token_client.balance(&contract_id), 30_000_000);

    let circle_after_b1 = client.get_circle(&circle_id);
    assert_eq!(circle_after_b1.current_volume, 3);
    assert_eq!(circle_after_b1.status, CircleStatus::Open); // Min is 5, so not quorum yet

    // 3. Buyer 2 commits 3 units (Cost = 3 * 1 = 3 XLM = 30_000_000)
    client.commit_demand(&buyer2, &circle_id, &3);
    assert_eq!(token_client.balance(&buyer2), 70_000_000);
    assert_eq!(token_client.balance(&contract_id), 60_000_000);

    let circle_after_b2 = client.get_circle(&circle_id);
    assert_eq!(circle_after_b2.current_volume, 6);
    assert_eq!(circle_after_b2.status, CircleStatus::QuorumReached); // 6 >= 5!

    // 4. Seller submits a competitive offer at 9_500_000 (0.95 XLM per unit for volume >= 6)
    let offer_id = client.submit_seller_offer(&seller, &circle_id, &9_500_000, &10);
    assert_eq!(offer_id, 1);

    let offer = client.get_offer(&offer_id);
    assert_eq!(offer.status, OfferStatus::Submitted);

    // 5. Settle deal with winning offer
    // Payout = 6 units * 9_500_000 = 57_000_000
    client.settle_deal(&circle_id, &offer_id);

    let settled_circle = client.get_circle(&circle_id);
    assert_eq!(settled_circle.status, CircleStatus::Settled);

    let winning_offer = client.get_offer(&offer_id);
    assert_eq!(winning_offer.status, OfferStatus::Accepted);

    // Verify seller received payout
    assert_eq!(token_client.balance(&seller), 57_000_000);
}

#[test]
fn test_demand_circle_refund_on_expiry() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = token::Client::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());

    token_admin_client.mint(&buyer, &50_000_000);

    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);

    client.initialize(&admin);

    // Create circle with 1000s duration
    let circle_id = client.create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &10,
        &50,
        &1000,
    );

    // Buyer commits 2 units (20_000_000)
    client.commit_demand(&buyer, &circle_id, &2);
    assert_eq!(token_client.balance(&buyer), 30_000_000);
    assert_eq!(token_client.balance(&contract_id), 20_000_000);

    // Fast-forward past deadline
    env.ledger().set_timestamp(2000);

    // Buyer claims refund
    client.claim_refund(&buyer, &circle_id);
    assert_eq!(token_client.balance(&buyer), 50_000_000);
    assert_eq!(token_client.balance(&contract_id), 0);
}
