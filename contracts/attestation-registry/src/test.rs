#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

fn setup() -> (Env, AttestationRegistryClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AttestationRegistry, ());
    let client = AttestationRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let attester_registry_id = env.register(attester_registry::AttesterRegistry, ());
    (env, client, admin, attester_registry_id)
}

#[test]
fn initialize_sets_admin_and_attester_registry() {
    let (_, client, admin, attester_registry_id) = setup();
    client.initialize(&admin, &attester_registry_id);
}

#[test]
fn initialize_twice_fails() {
    let (_, client, admin, attester_registry_id) = setup();
    client.initialize(&admin, &attester_registry_id);

    let result = client.try_initialize(&admin, &attester_registry_id);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}
