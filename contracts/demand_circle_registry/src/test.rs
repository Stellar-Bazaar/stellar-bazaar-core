#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

#[test]
fn test_registry_initialization_and_creation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    let contract_id = env.register(DemandCircleRegistryContract, ());
    let client = DemandCircleRegistryContractClient::new(&env, &contract_id);

    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_circle_count(), 0);

    // Create circle 1
    let title = String::from_str(&env, "Solar Inverters 5kW");
    let uri = String::from_str(&env, "ipfs://bafybeibazaar1");
    let id1 = client.create_circle(
        &creator,
        &title,
        &uri,
        &50,          // 50 units
        &120_000_000, // 12 XLM per unit in stroops
        &86400,       // 24 hours
    );
    assert_eq!(id1, 1);
    assert_eq!(client.get_circle_count(), 1);

    let circle1 = client.get_circle(&id1);
    assert_eq!(circle1.id, 1);
    assert_eq!(circle1.creator, creator);
    assert_eq!(circle1.title, title);
    assert_eq!(circle1.target_quantity, 50);
    assert_eq!(circle1.target_price_stroops, 120_000_000);
    assert_eq!(circle1.status, CircleStatus::Open);

    // Create circle 2
    let title2 = String::from_str(&env, "AgriTech Soil Sensors");
    let uri2 = String::from_str(&env, "ipfs://bafybeibazaar2");
    let id2 = client.create_circle(&creator, &title2, &uri2, &100, &45_000_000, &172800);
    assert_eq!(id2, 2);
    assert_eq!(client.get_circle_count(), 2);
}

#[test]
fn test_validation_rules_reject_invalid_inputs() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    let contract_id = env.register(DemandCircleRegistryContract, ());
    let client = DemandCircleRegistryContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let valid_title = String::from_str(&env, "Test Circle");
    let valid_uri = String::from_str(&env, "ipfs://test");

    // Zero quantity should fail
    let res_zero_qty =
        client.try_create_circle(&creator, &valid_title, &valid_uri, &0, &10_000_000, &86400);
    assert_eq!(res_zero_qty, Err(Ok(Error::InvalidQuantity)));

    // Zero or negative price should fail
    let res_zero_price =
        client.try_create_circle(&creator, &valid_title, &valid_uri, &10, &0, &86400);
    assert_eq!(res_zero_price, Err(Ok(Error::InvalidPrice)));

    // Too short deadline (< 60s) should fail
    let res_short_deadline =
        client.try_create_circle(&creator, &valid_title, &valid_uri, &10, &10_000_000, &30);
    assert_eq!(res_short_deadline, Err(Ok(Error::InvalidDeadline)));
}

#[test]
fn test_creator_close_and_unauthorized_rejection() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let impostor = Address::generate(&env);

    let contract_id = env.register(DemandCircleRegistryContract, ());
    let client = DemandCircleRegistryContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let title = String::from_str(&env, "Coffee Beans Bulk");
    let uri = String::from_str(&env, "ipfs://coffee");
    let id = client.create_circle(&creator, &title, &uri, &20, &50_000_000, &86400);

    // Impostor attempts to close circle -> Unauthorized
    let res_impostor = client.try_close_circle(&impostor, &id);
    assert_eq!(res_impostor, Err(Ok(Error::Unauthorized)));

    // Legitimate creator closes circle
    client.close_circle(&creator, &id);
    let closed_circle = client.get_circle(&id);
    assert_eq!(closed_circle.status, CircleStatus::Closed);

    // Cannot close already closed circle
    let res_already_closed = client.try_close_circle(&creator, &id);
    assert_eq!(res_already_closed, Err(Ok(Error::InvalidState)));
}

#[test]
fn test_circle_expiration_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    let contract_id = env.register(DemandCircleRegistryContract, ());
    let client = DemandCircleRegistryContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    let title = String::from_str(&env, "Time-bounded Pool");
    let uri = String::from_str(&env, "ipfs://time");
    let id = client.create_circle(&creator, &title, &uri, &10, &10_000_000, &3600); // 1 hour

    // Attempt to expire immediately -> DeadlineNotPassed
    let res_premature = client.try_expire_circle(&id);
    assert_eq!(res_premature, Err(Ok(Error::DeadlineNotPassed)));

    // Fast-forward ledger timestamp past deadline (3601s)
    env.ledger().set_timestamp(4000);

    // Expire circle
    client.expire_circle(&id);
    let expired_circle = client.get_circle(&id);
    assert_eq!(expired_circle.status, CircleStatus::Expired);
}
