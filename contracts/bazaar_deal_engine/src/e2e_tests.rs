#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env,
};

/// Test fixture bundling deal engine client, SAC tokens, and test actor accounts.
#[allow(dead_code)]
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

#[test]
fn test_e2e_happy_path_cooperative_deal_lifecycle() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    // 1. Creator launches Demand Circle
    // Target price = 10_000_000 stroops (1 XLM), min volume = 10, max volume = 50
    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &10,
        &50,
        &86_400,
    );
    assert_eq!(circle_id, 1);

    // 2. Three buyers commit 5 units each (Total 15 units, reaching quorum >= 10)
    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[1], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[2], &circle_id, &5);

    let circle = fix.deal_client.get_circle(&circle_id);
    assert_eq!(circle.current_volume, 15);
    assert_eq!(circle.status, CircleStatus::QuorumReached);
    assert_eq!(fix.token_client.balance(&fix.contract_id), 150_000_000); // 15 * 10 XLM

    // 3. Merchant places wholesale offer at 8_000_000 stroops (20% savings, 30 units capacity, 7 day lead time)
    let offer_id = fix.deal_client.submit_seller_offer(
        &fix.seller,
        &circle_id,
        &8_000_000,
        &30,
        &7,
    );
    assert_eq!(offer_id, 1);

    // 4. Circle creator accepts merchant offer
    fix.deal_client.accept_seller_offer(&fix.creator, &circle_id, &offer_id);

    // 5. Creator initiates inspection period for 24 hours (86,400s)
    fix.deal_client.initiate_inspection(&fix.creator, &circle_id, &86_400);
    let circle_inspecting = fix.deal_client.get_circle(&circle_id);
    assert_eq!(circle_inspecting.status, CircleStatus::Inspection);

    // 6. Creator confirms inspection and delivery; escrow settles
    fix.deal_client.confirm_inspection_and_settle(&fix.creator, &circle_id);

    let settled_circle = fix.deal_client.get_circle(&circle_id);
    assert_eq!(settled_circle.status, CircleStatus::Settled);

    // 7. Verify balances after settlement:
    // Wholesale volume: 15 * 8_000_000 = 120_000_000
    // Protocol fee (1%): 1_200_000 stroops to Treasury
    // Net to seller: 118_800_000 stroops
    assert_eq!(fix.token_client.balance(&fix.seller), 118_800_000);
    assert_eq!(fix.token_client.balance(&fix.treasury), 1_200_000);

    // Remaining in contract: 30_000_000 stroops reserved for buyer surplus rebates
    assert_eq!(fix.token_client.balance(&fix.contract_id), 30_000_000);
}

#[test]
fn test_e2e_surplus_rebate_distribution() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &10,
        &50,
        &86_400,
    );

    // 3 buyers commit 5 units each (deposit 50_000_000 each)
    // Starting balance of each buyer: 1_000_000_000
    // Balance after commitment: 950_000_000
    for b in &fix.buyers {
        fix.deal_client.commit_demand(b, &circle_id, &5);
        assert_eq!(fix.token_client.balance(b), 950_000_000);
    }

    let offer_id = fix.deal_client.submit_seller_offer(
        &fix.seller,
        &circle_id,
        &8_000_000,
        &30,
        &7,
    );

    fix.deal_client.accept_seller_offer(&fix.creator, &circle_id, &offer_id);
    fix.deal_client.initiate_inspection(&fix.creator, &circle_id, &86_400);
    fix.deal_client.confirm_inspection_and_settle(&fix.creator, &circle_id);

    // Surplus in contract: 30_000_000 stroops
    assert_eq!(fix.token_client.balance(&fix.contract_id), 30_000_000);

    // Each buyer claims their 10_000_000 stroops surplus rebate
    for b in &fix.buyers {
        fix.deal_client.claim_surplus_rebate(b, &circle_id);
        // Buyer balance restored by 10_000_000 -> 960_000_000 (effective cost = 40_000_000)
        assert_eq!(fix.token_client.balance(b), 960_000_000);
    }

    // After all 3 buyers claim, contract escrow holds 0 stroops
    assert_eq!(fix.token_client.balance(&fix.contract_id), 0);
}

#[test]
fn test_e2e_buyer_dispute_filing_during_inspection() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &10,
        &50,
        &86_400,
    );

    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[1], &circle_id, &5);

    let offer_id = fix.deal_client.submit_seller_offer(
        &fix.seller,
        &circle_id,
        &9_000_000,
        &20,
        &5,
    );

    fix.deal_client.accept_seller_offer(&fix.creator, &circle_id, &offer_id);
    fix.deal_client.initiate_inspection(&fix.creator, &circle_id, &86_400);

    // Buyer 0 notices items do not meet specifications and files dispute
    let reason = symbol_short!("damaged");
    fix.deal_client.dispute_deal(&fix.buyers[0], &circle_id, &reason);

    let circle = fix.deal_client.get_circle(&circle_id);
    assert_eq!(circle.status, CircleStatus::Disputed);

    // Settlement cannot proceed while under dispute
    let err = fix.deal_client.try_confirm_inspection_and_settle(&fix.creator, &circle_id);
    assert_eq!(err, Err(Ok(Error::NotUnderInspection)));
}

#[test]
fn test_e2e_arbitrator_dispute_resolution() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &10,
        &50,
        &86_400,
    );

    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[1], &circle_id, &5);

    let offer_id = fix.deal_client.submit_seller_offer(
        &fix.seller,
        &circle_id,
        &8_500_000,
        &20,
        &5,
    );

    fix.deal_client.accept_seller_offer(&fix.creator, &circle_id, &offer_id);
    fix.deal_client.initiate_inspection(&fix.creator, &circle_id, &86_400);

    // Buyer files dispute
    fix.deal_client.dispute_deal(&fix.buyers[0], &circle_id, &symbol_short!("damaged"));

    // Protocol admin arbitrates and resolves in favor of buyers (refund_to_buyers = true)
    fix.deal_client.resolve_dispute(&fix.admin, &circle_id, &true);

    let circle = fix.deal_client.get_circle(&circle_id);
    assert_eq!(circle.status, CircleStatus::Cancelled);

    // Verify seller reputation now records disputed deal
    let rep = fix.deal_client.get_seller_reputation(&fix.seller);
    assert_eq!(rep.disputed_or_refunded_deals, 1);
    assert_eq!(rep.successful_deals, 0);

    // Buyer can refund their escrow deposit
    fix.deal_client.claim_refund(&fix.buyers[0], &circle_id);
    assert_eq!(fix.token_client.balance(&fix.buyers[0]), 1_000_000_000);
}

#[test]
fn test_e2e_unauthorized_dispute_resolution_rejection() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &10,
        &50,
        &86_400,
    );

    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[1], &circle_id, &5);

    let offer_id = fix.deal_client.submit_seller_offer(
        &fix.seller,
        &circle_id,
        &8_500_000,
        &20,
        &5,
    );

    fix.deal_client.accept_seller_offer(&fix.creator, &circle_id, &offer_id);
    fix.deal_client.initiate_inspection(&fix.creator, &circle_id, &86_400);

    fix.deal_client.dispute_deal(&fix.buyers[0], &circle_id, &symbol_short!("damaged"));

    // Unauthorized non-admin attempts resolution
    let unauthorized_caller = Address::generate(&env);
    let err = fix.deal_client.try_resolve_dispute(&unauthorized_caller, &circle_id, &true);
    assert_eq!(err, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_e2e_double_claim_rebate_protection() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &10,
        &50,
        &86_400,
    );

    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[1], &circle_id, &5);

    let offer_id = fix.deal_client.submit_seller_offer(
        &fix.seller,
        &circle_id,
        &8_000_000,
        &20,
        &5,
    );

    fix.deal_client.accept_seller_offer(&fix.creator, &circle_id, &offer_id);
    fix.deal_client.initiate_inspection(&fix.creator, &circle_id, &86_400);
    fix.deal_client.confirm_inspection_and_settle(&fix.creator, &circle_id);

    // First claim succeeds
    fix.deal_client.claim_surplus_rebate(&fix.buyers[0], &circle_id);

    // Second claim must fail with AlreadyClaimedRebate
    let err = fix.deal_client.try_claim_surplus_rebate(&fix.buyers[0], &circle_id);
    assert_eq!(err, Err(Ok(Error::AlreadyClaimedRebate)));
}

#[test]
fn test_e2e_expired_circle_refund() {
    use soroban_sdk::testutils::Ledger as _;

    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &10,
        &50,
        &100, // 100 seconds
    );

    // Buyer 0 commits 3 units (30_000_000 stroops). Quorum not reached.
    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &3);
    assert_eq!(fix.token_client.balance(&fix.buyers[0]), 970_000_000);

    // Fast-forward ledger timestamp past circle deadline
    env.ledger().set_timestamp(env.ledger().timestamp() + 500);

    // Buyer claims refund
    fix.deal_client.claim_refund(&fix.buyers[0], &circle_id);

    // Buyer receives full 30_000_000 stroops back
    assert_eq!(fix.token_client.balance(&fix.buyers[0]), 1_000_000_000);
    assert_eq!(fix.token_client.balance(&fix.contract_id), 0);
}

#[test]
fn test_e2e_premature_settlement_rejection() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &10,
        &50,
        &86_400,
    );

    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[1], &circle_id, &5);

    let offer_id = fix.deal_client.submit_seller_offer(
        &fix.seller,
        &circle_id,
        &8_000_000,
        &20,
        &5,
    );

    fix.deal_client.accept_seller_offer(&fix.creator, &circle_id, &offer_id);

    // Circle is Accepted but NOT yet in Inspection phase
    let circle = fix.deal_client.get_circle(&circle_id);
    assert_eq!(circle.status, CircleStatus::QuorumReached);

    // Calling confirm_inspection_and_settle prematurely must fail with NotUnderInspection
    let err = fix.deal_client.try_confirm_inspection_and_settle(&fix.creator, &circle_id);
    assert_eq!(err, Err(Ok(Error::NotUnderInspection)));
}

#[test]
fn test_e2e_offer_price_exceeds_target_rejection() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000, // 1.0 XLM ceiling
        &10,
        &50,
        &86_400,
    );

    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[1], &circle_id, &5);

    // Offer price 12_000_000 > target ceiling 10_000_000
    let err = fix.deal_client.try_submit_seller_offer(
        &fix.seller,
        &circle_id,
        &12_000_000,
        &20,
        &5,
    );

    assert_eq!(err, Err(Ok(Error::PriceExceedsTarget)));
}

#[test]
fn test_e2e_duplicate_merchant_offer_rejection() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &10,
        &50,
        &86_400,
    );

    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[1], &circle_id, &5);

    // First offer succeeds
    let offer_id = fix.deal_client.submit_seller_offer(
        &fix.seller,
        &circle_id,
        &9_000_000,
        &20,
        &5,
    );
    assert_eq!(offer_id, 1);

    // Second offer from the same seller for the same circle fails
    let err = fix.deal_client.try_submit_seller_offer(
        &fix.seller,
        &circle_id,
        &8_500_000,
        &25,
        &4,
    );
    assert_eq!(err, Err(Ok(Error::DuplicateOffer)));
}

#[test]
fn test_e2e_maximum_volume_overflow_enforcement() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &fix.token_address,
        &10_000_000,
        &5,
        &15, // max volume = 15
        &86_400,
    );

    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &10);

    // Attempting to commit 10 more units exceeds 15 max volume cap
    let err = fix.deal_client.try_commit_demand(&fix.buyers[1], &circle_id, &10);
    assert_eq!(err, Err(Ok(Error::VolumeExceeded)));
}

#[test]
fn test_e2e_invalid_protocol_fee_bps_rejection() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    // Protocol maximum fee is 1,000 bps (10%)
    // Setting 1,001 bps must fail with InvalidFeeBps
    let err = fix.deal_client.try_set_protocol_fee(&fix.admin, &1_001);
    assert_eq!(err, Err(Ok(Error::InvalidFeeBps)));

    // Setting allowable fee 250 bps succeeds
    let res = fix.deal_client.try_set_protocol_fee(&fix.admin, &250);
    assert!(res.is_ok());
    assert_eq!(fix.deal_client.get_protocol_fee_bps(), 250);
}

#[test]
fn test_e2e_multi_asset_sac_settlement() {
    let env = Env::default();
    let fix = E2EFixture::new(&env);

    // Deploy an independent USDC SAC token contract
    let usdc_admin = Address::generate(&env);
    let usdc_contract = env.register_stellar_asset_contract_v2(usdc_admin);
    let usdc_address = usdc_contract.address();
    let usdc_token_client = token::Client::new(&env, &usdc_address);
    let usdc_admin_client = StellarAssetClient::new(&env, &usdc_address);

    // Mint 500 USDC (5,000,000,000 stroops) to buyers
    usdc_admin_client.mint(&fix.buyers[0], &5_000_000_000);
    usdc_admin_client.mint(&fix.buyers[1], &5_000_000_000);

    // 1. Create Demand Circle for USDC
    // Target unit price: 10_000_000 (1 USDC), min volume 10, max volume 50
    let circle_id = fix.deal_client.create_circle(
        &fix.creator,
        &usdc_address,
        &10_000_000,
        &10,
        &50,
        &86_400,
    );

    // 2. Commit USDC demand
    fix.deal_client.commit_demand(&fix.buyers[0], &circle_id, &5);
    fix.deal_client.commit_demand(&fix.buyers[1], &circle_id, &5);

    assert_eq!(usdc_token_client.balance(&fix.contract_id), 100_000_000);
    // Base XLM token contract balance remains 0
    assert_eq!(fix.token_client.balance(&fix.contract_id), 0);

    // 3. Wholesale offer in USDC at 9 USDC
    let offer_id = fix.deal_client.submit_seller_offer(
        &fix.seller,
        &circle_id,
        &9_000_000,
        &20,
        &5,
    );

    fix.deal_client.accept_seller_offer(&fix.creator, &circle_id, &offer_id);
    fix.deal_client.initiate_inspection(&fix.creator, &circle_id, &86_400);
    fix.deal_client.confirm_inspection_and_settle(&fix.creator, &circle_id);

    // 4. Verify wholesale payout in USDC
    // 10 units * 9 USDC = 90_000_000 stroops
    // Fee 1%: 900_000 stroops
    // Seller net: 89_100_000 USDC stroops
    assert_eq!(usdc_token_client.balance(&fix.seller), 89_100_000);
    assert_eq!(usdc_token_client.balance(&fix.treasury), 900_000);
}
