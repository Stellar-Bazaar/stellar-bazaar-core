#![cfg(test)]

use super::*;
use demand_circle_registry::DemandCircleRegistryContract;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String,
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
    let offer_id = client.submit_seller_offer(&seller, &circle_id, &9_500_000, &10, &7);
    assert_eq!(offer_id, 1);

    let offer = client.get_offer(&offer_id);
    assert_eq!(offer.status, OfferStatus::Submitted);
    assert_eq!(offer.lead_time_days, 7);

    let circle_offers = client.get_circle_offers(&circle_id);
    assert_eq!(circle_offers.len(), 1);
    assert_eq!(circle_offers.get(0).unwrap(), offer_id);

    // 5. Accept seller offer
    client.accept_seller_offer(&creator, &circle_id, &offer_id);

    // 6. Settle deal with winning offer
    // Payout = 6 units * 9_500_000 = 57_000_000
    client.settle_deal(&circle_id, &offer_id);

    let settled_circle = client.get_circle(&circle_id);
    assert_eq!(settled_circle.status, CircleStatus::Settled);
    assert_eq!(settled_circle.accepted_offer_id, Some(offer_id));

    let winning_offer = client.get_offer(&offer_id);
    assert_eq!(winning_offer.status, OfferStatus::Accepted);

    // Verify seller received net payout (57_000_000 - 1% protocol fee 570_000 = 56_430_000)
    assert_eq!(token_client.balance(&seller), 56_430_000);
    // Verify protocol treasury received fee
    assert_eq!(token_client.balance(&admin), 570_000);

    // Verify verifiable seller reputation record was updated on-chain
    let rep = client.get_seller_reputation(&seller);
    assert_eq!(rep.successful_deals, 1);
    assert_eq!(rep.total_volume_settled, 6);
    assert_eq!(rep.total_amount_settled, 57_000_000);
    assert_eq!(rep.disputed_or_refunded_deals, 0);
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

    // Duplicate refund must fail
    let dup_res = client.try_claim_refund(&buyer, &circle_id);
    assert!(dup_res.is_err());
}

#[test]
fn test_cross_contract_registry_deal_registration() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    // 1. Deploy DemandCircleRegistryContract
    let registry_id = env.register(DemandCircleRegistryContract, ());
    let registry_client =
        demand_circle_registry::DemandCircleRegistryContractClient::new(&env, &registry_id);
    registry_client.initialize(&admin);

    // 2. Create circle on Registry
    let reg_circle_id = registry_client.create_circle(
        &creator,
        &String::from_str(&env, "Bulk Solar Panels 400W"),
        &String::from_str(&env, "ipfs://bafybeibazaar1"),
        &15,
        &20_000_000, // 2 XLM per panel
        &86400,
    );
    assert_eq!(reg_circle_id, 1);

    // 3. Register Deal Engine Contract
    let engine_id = env.register(BazaarDealEngineContract, ());
    let engine_client = BazaarDealEngineContractClient::new(&env, &engine_id);
    engine_client.initialize(&admin);

    // Token
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    // 4. Deal Engine performs genuine Cross-Contract call to Registry Contract
    let deal_id = engine_client.create_deal_from_registry(
        &creator,
        &registry_id,
        &reg_circle_id,
        &token_contract.address(),
        &5,
        &15,
    );
    assert_eq!(deal_id, 1);

    // Verify Deal Engine loaded authoritative constraints from Registry
    let deal = engine_client.get_circle(&deal_id);
    assert_eq!(deal.target_unit_price, 20_000_000);
    assert_eq!(deal.max_volume, 15);
    assert_eq!(deal.min_volume, 5);
    assert_eq!(deal.registry_address, Some(registry_id.clone()));
    assert_eq!(deal.registry_circle_id, Some(1));

    // 5. Unauthorized creator cross-contract attempt fails
    let imposter = Address::generate(&env);
    let unauth_res = engine_client.try_create_deal_from_registry(
        &imposter,
        &registry_id,
        &reg_circle_id,
        &token_contract.address(),
        &5,
        &15,
    );
    assert!(unauth_res.is_err());
}

#[test]
fn test_duplicate_commitment_and_volume_overflow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());
    token_admin_client.mint(&buyer, &100_000_000);

    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let circle_id = client.create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &2,
        &5,
        &86400,
    );

    // Valid commitment
    client.commit_demand(&buyer, &circle_id, &3);

    // Duplicate commitment from same buyer rejected
    let dup_res = client.try_commit_demand(&buyer, &circle_id, &1);
    assert!(dup_res.is_err());

    // Another buyer tries to exceed max_volume (3 + 3 = 6 > 5)
    let buyer2 = Address::generate(&env);
    token_admin_client.mint(&buyer2, &100_000_000);
    let overflow_res = client.try_commit_demand(&buyer2, &circle_id, &3);
    assert!(overflow_res.is_err());
}

#[test]
fn test_seller_offer_validation_and_cancellation_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = token::Client::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());
    token_admin_client.mint(&buyer, &50_000_000);

    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let circle_id = client.create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &2,
        &10,
        &86400,
    );

    client.commit_demand(&buyer, &circle_id, &2);

    // Offer exceeding target price rejected
    let bad_offer_res = client.try_submit_seller_offer(&seller, &circle_id, &11_000_000, &5, &3);
    assert!(bad_offer_res.is_err());

    // Creator cancels circle
    client.cancel_circle(&creator, &circle_id);

    // Buyer claims full refund on cancelled circle
    client.claim_refund(&buyer, &circle_id);
    assert_eq!(token_client.balance(&buyer), 50_000_000);
}


#[test]
fn test_deal_creation_zero_target_volume_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    // min_volume = 0 should return Err(Error::InvalidVolume)
    let res_zero_vol = client.try_create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &0, // invalid 0 volume
        &10,
        &86400,
    );
    assert_eq!(res_zero_vol, Err(Ok(Error::InvalidVolume)));
}

#[test]
fn test_duplicate_seller_offer_rejection() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let seller = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);

    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let circle_id = client.create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &5,
        &20,
        &86400,
    );

    // Initial offer from seller
    let offer_id = client.submit_seller_offer(&seller, &circle_id, &9_000_000, &10, &5);
    assert_eq!(offer_id, 1);

    // Second active offer from the same seller for the same circle should be rejected as DuplicateOffer
    let dup_offer = client.try_submit_seller_offer(&seller, &circle_id, &8_500_000, &10, &4);
    assert_eq!(dup_offer, Err(Ok(Error::DuplicateOffer)));
}

#[test]
fn test_buyer_refund_on_creator_cancellation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = token::Client::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());
    token_admin_client.mint(&buyer, &80_000_000);

    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let circle_id = client.create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &5,
        &20,
        &86400,
    );

    // Buyer commits 4 units = 40_000_000 stroops
    client.commit_demand(&buyer, &circle_id, &4);
    assert_eq!(token_client.balance(&buyer), 40_000_000);
    assert_eq!(token_client.balance(&contract_id), 40_000_000);

    // Creator cancels the circle
    client.cancel_circle(&creator, &circle_id);
    let circle = client.get_circle(&circle_id);
    assert_eq!(circle.status, CircleStatus::Cancelled);

    // Buyer claims refund and gets 100% of escrow back
    client.claim_refund(&buyer, &circle_id);
    assert_eq!(token_client.balance(&buyer), 80_000_000);
    assert_eq!(token_client.balance(&contract_id), 0);

    // Cannot claim refund a second time
    let repeat_refund = client.try_claim_refund(&buyer, &circle_id);
    assert_eq!(repeat_refund, Err(Ok(Error::AlreadyRefunded)));
}

#[test]
fn test_deal_inspection_lifecycle_and_settlement() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = token::Client::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());
    token_admin_client.mint(&buyer, &100_000_000);

    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let circle_id = client.create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &5,
        &20,
        &86400,
    );

    // Buyer commits 5 units, reaching quorum
    client.commit_demand(&buyer, &circle_id, &5);
    let circle_pre = client.get_circle(&circle_id);
    assert_eq!(circle_pre.status, CircleStatus::QuorumReached);

    // Seller submits offer
    let offer_id = client.submit_seller_offer(&seller, &circle_id, &9_000_000, &10, &3);
    client.accept_seller_offer(&creator, &circle_id, &offer_id);

    // Attempting settlement before inspection initiated should fail if called via confirm_inspection_and_settle
    let err_premature = client.try_confirm_inspection_and_settle(&creator, &circle_id);
    assert_eq!(err_premature, Err(Ok(Error::NotUnderInspection)));

    // Creator initiates inspection period for 3 days (259,200s)
    client.initiate_inspection(&creator, &circle_id, &259_200);
    let circle_inspect = client.get_circle(&circle_id);
    assert_eq!(circle_inspect.status, CircleStatus::Inspection);

    // Unauthorized user cannot confirm delivery
    let err_unauth = client.try_confirm_inspection_and_settle(&unauthorized_user, &circle_id);
    assert_eq!(err_unauth, Err(Ok(Error::Unauthorized)));

    // Creator confirms inspection and delivery; escrow settles
    client.confirm_inspection_and_settle(&creator, &circle_id);
    let circle_settled = client.get_circle(&circle_id);
    assert_eq!(circle_settled.status, CircleStatus::Settled);

    // Verify seller received 44_550_000 stroops (45_000_000 gross minus 1% fee 450_000)
    assert_eq!(token_client.balance(&seller), 44_550_000);
    // Verify admin treasury received protocol fee
    assert_eq!(token_client.balance(&admin), 450_000);
}

#[test]
fn test_dispute_deal_initiation_and_authorization_guards() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    let stranger = Address::generate(&env);
    let seller = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());
    token_admin_client.mint(&buyer, &100_000_000);

    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let circle_id = client.create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &5,
        &20,
        &86400,
    );

    client.commit_demand(&buyer, &circle_id, &5);
    let offer_id = client.submit_seller_offer(&seller, &circle_id, &9_000_000, &10, &3);
    client.accept_seller_offer(&creator, &circle_id, &offer_id);
    client.initiate_inspection(&creator, &circle_id, &86400);

    // Stranger without commitment cannot file dispute
    let unauth_dispute = client.try_dispute_deal(&stranger, &circle_id, &Symbol::new(&env, "DEFECT"));
    assert_eq!(unauth_dispute, Err(Ok(Error::DisputeUnauthorized)));

    // Committed buyer can file dispute
    client.dispute_deal(&buyer, &circle_id, &Symbol::new(&env, "DAMAGED_GOODS"));
    let circle = client.get_circle(&circle_id);
    assert_eq!(circle.status, CircleStatus::Disputed);

    // Cannot file dispute again while already disputed
    let dup_dispute = client.try_dispute_deal(&creator, &circle_id, &Symbol::new(&env, "LATE"));
    assert_eq!(dup_dispute, Err(Ok(Error::DisputeAlreadyOpen)));
}

#[test]
fn test_dispute_resolution_with_buyer_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = token::Client::new(&env, &token_contract.address());
    let token_admin_client = StellarAssetClient::new(&env, &token_contract.address());
    token_admin_client.mint(&buyer, &100_000_000);

    let contract_id = env.register(BazaarDealEngineContract, ());
    let client = BazaarDealEngineContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let circle_id = client.create_circle(
        &creator,
        &token_contract.address(),
        &10_000_000,
        &5,
        &20,
        &86400,
    );

    client.commit_demand(&buyer, &circle_id, &5);
    let offer_id = client.submit_seller_offer(&seller, &circle_id, &9_000_000, &10, &3);
    client.accept_seller_offer(&creator, &circle_id, &offer_id);
    client.initiate_inspection(&creator, &circle_id, &86400);

    client.dispute_deal(&buyer, &circle_id, &Symbol::new(&env, "NON_CONFORMING"));

    // Unauthorized non-admin cannot resolve dispute
    let unauth_res = client.try_resolve_dispute(&non_admin, &circle_id, &true);
    assert_eq!(unauth_res, Err(Ok(Error::Unauthorized)));

    // Admin resolves dispute in favor of buyers (refund_to_buyers: true)
    client.resolve_dispute(&admin, &circle_id, &true);
    let circle = client.get_circle(&circle_id);
    assert_eq!(circle.status, CircleStatus::Cancelled);

    // Buyer reclaims 100% refund
    client.claim_refund(&buyer, &circle_id);
    assert_eq!(token_client.balance(&buyer), 100_000_000);

    // Seller reputation tracks disputed deal penalty
    let rep = client.get_seller_reputation(&seller);
    assert_eq!(rep.disputed_or_refunded_deals, 1);
    assert_eq!(rep.successful_deals, 0);
}
