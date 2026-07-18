#![cfg(test)]

/*
Authentication/Authorization Matrix Table:

| Function        | Caller Case        | Arg / Call Target | Auth Address | Expected Result / Error Variant |
|-----------------|--------------------|-------------------|--------------|---------------------------------|
| initialize      | Right Caller       | admin             | admin        | Ok(())                          |
| initialize      | Wrong Caller       | admin             | wrong_user   | Err(Err(InvokeError::Abort))    |
| initialize      | No Auth            | admin             | None         | Err(Err(InvokeError::Abort))    |
| initialize      | Role Confusion     | admin             | attester     | Err(Err(InvokeError::Abort))    |
| add_attester    | Right Caller       | attester          | admin        | Ok(())                          |
| add_attester    | Wrong Caller       | attester          | wrong_user   | Err(Err(InvokeError::Abort))    |
| add_attester    | No Auth            | attester          | None         | Err(Err(InvokeError::Abort))    |
| add_attester    | Role Confusion     | attester          | attester     | Err(Err(InvokeError::Abort))    |
| remove_attester | Right Caller       | attester          | admin        | Ok(())                          |
| remove_attester | Wrong Caller       | attester          | wrong_user   | Err(Err(InvokeError::Abort))    |
| remove_attester | No Auth            | attester          | None         | Err(Err(InvokeError::Abort))    |
| remove_attester | Role Confusion     | attester          | attester     | Err(Err(InvokeError::Abort))    |
*/

extern crate std;

use super::*;
use soroban_sdk::testutils::{Address as _, Events as _};
use soroban_sdk::{Env, Event, IntoVal};

fn setup() -> (Env, AttesterRegistryClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(AttesterRegistry, ());
    let client = AttesterRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    (env, client, admin)
}

#[test]
fn initialize_sets_admin() {
    let (_, client, admin) = setup();
    client.initialize(&admin);
}

#[test]
fn initialize_twice_fails() {
    let (_, client, admin) = setup();
    client.initialize(&admin);

    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn is_attester_false_before_allowlisting() {
    let (env, client, admin) = setup();
    client.initialize(&admin);

    let someone = Address::generate(&env);
    assert!(!client.is_attester(&someone));
}

#[test]
fn add_attester_allowlists_and_emits_event() {
    let (env, client, admin) = setup();
    client.initialize(&admin);

    let attester = Address::generate(&env);
    client.add_attester(&attester);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            soroban_sdk::testutils::AuthorizedInvocation {
                function: soroban_sdk::testutils::AuthorizedFunction::Contract((
                    client.address.clone(),
                    soroban_sdk::Symbol::new(&env, "add_attester"),
                    (attester.clone(),).into_val(&env),
                )),
                sub_invocations: std::vec![],
            },
        )]
    );

    let expected_event = AttesterAdded {
        attester: attester.clone(),
    };
    assert_eq!(
        env.events().all(),
        std::vec![expected_event.to_xdr(&env, &client.address)],
    );

    assert!(client.is_attester(&attester));
}

#[test]
fn remove_attester_revokes_allowlisting() {
    let (env, client, admin) = setup();
    client.initialize(&admin);

    let attester = Address::generate(&env);
    client.add_attester(&attester);
    assert!(client.is_attester(&attester));

    client.remove_attester(&attester);
    assert!(!client.is_attester(&attester));
}

#[test]
fn remove_attester_never_added_is_a_no_op() {
    let (env, client, admin) = setup();
    client.initialize(&admin);

    let attester = Address::generate(&env);
    client.remove_attester(&attester);
    assert!(!client.is_attester(&attester));
}

#[test]
fn add_attester_before_initialize_fails() {
    let (env, client, _admin) = setup();
    let attester = Address::generate(&env);

    let result = client.try_add_attester(&attester);
    assert_eq!(result, Err(Ok(Error::NotInitialized)));
}

#[test]
fn add_attester_without_admin_auth_fails() {
    // No mock_all_auths(): calls must present a real, matching auth entry.
    let env = Env::default();
    let contract_id = env.register(AttesterRegistry, ());
    let client = AttesterRegistryClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let attester = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);

    // Only mock an auth entry for `attester`, not `admin`, so the
    // contract's `admin.require_auth()` has nothing to satisfy it.
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &attester,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &client.address,
            fn_name: "add_attester",
            args: (attester.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    let result = client.try_add_attester(&attester);
    assert_eq!(result, Err(Err(soroban_sdk::InvokeError::Abort)));
    assert!(!client.is_attester(&attester));
}

#[test]
fn test_initialize_auth_matrix() {

    struct TestCase {
        name: &'static str,
        auth_role: &'static str, // "admin", "wrong_user", "none", "attester"
        expected_result: Result<Result<(), soroban_sdk::ConversionError>, Result<Error, soroban_sdk::InvokeError>>,
    }

    let cases = std::vec![
        TestCase {
            name: "Right Caller (Admin)",
            auth_role: "admin",
            expected_result: Ok(Ok(())),
        },
        TestCase {
            name: "Wrong Caller (Wrong User)",
            auth_role: "wrong_user",
            expected_result: Err(Err(soroban_sdk::InvokeError::Abort)),
        },
        TestCase {
            name: "No Auth Provided",
            auth_role: "none",
            expected_result: Err(Err(soroban_sdk::InvokeError::Abort)),
        },
        TestCase {
            name: "Role Confusion (Attester)",
            auth_role: "attester",
            expected_result: Err(Err(soroban_sdk::InvokeError::Abort)),
        },
    ];

    for case in cases {
        let case_env = Env::default();
        let case_contract_id = case_env.register(AttesterRegistry, ());
        let case_client = AttesterRegistryClient::new(&case_env, &case_contract_id);

        let case_admin = Address::generate(&case_env);
        let case_wrong_user = Address::generate(&case_env);
        let case_attester = Address::generate(&case_env);

        let auth_address = match case.auth_role {
            "admin" => Some(case_admin.clone()),
            "wrong_user" => Some(case_wrong_user.clone()),
            "attester" => Some(case_attester.clone()),
            _ => None,
        };

        if let Some(addr) = auth_address {
            case_env.mock_auths(&[soroban_sdk::testutils::MockAuth {
                address: &addr,
                invoke: &soroban_sdk::testutils::MockAuthInvoke {
                    contract: &case_client.address,
                    fn_name: "initialize",
                    args: (case_admin.clone(),).into_val(&case_env),
                    sub_invokes: &[],
                },
            }]);
        } else {
            case_env.mock_auths(&[]);
        }

        let result = case_client.try_initialize(&case_admin);
        assert_eq!(
            result, case.expected_result,
            "Failed case '{}': expected {:?}, got {:?}",
            case.name, case.expected_result, result
        );
    }
}

#[test]
fn test_add_attester_auth_matrix() {
    struct TestCase {
        name: &'static str,
        auth_role: &'static str, // "admin", "wrong_user", "none", "attester"
        expected_result: Result<Result<(), soroban_sdk::ConversionError>, Result<Error, soroban_sdk::InvokeError>>,
    }

    let cases = std::vec![
        TestCase {
            name: "Right Caller (Admin)",
            auth_role: "admin",
            expected_result: Ok(Ok(())),
        },
        TestCase {
            name: "Wrong Caller (Wrong User)",
            auth_role: "wrong_user",
            expected_result: Err(Err(soroban_sdk::InvokeError::Abort)),
        },
        TestCase {
            name: "No Auth Provided",
            auth_role: "none",
            expected_result: Err(Err(soroban_sdk::InvokeError::Abort)),
        },
        TestCase {
            name: "Role Confusion (Attester)",
            auth_role: "attester",
            expected_result: Err(Err(soroban_sdk::InvokeError::Abort)),
        },
    ];

    for case in cases {
        let env = Env::default();
        let contract_id = env.register(AttesterRegistry, ());
        let client = AttesterRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let wrong_user = Address::generate(&env);
        let attester = Address::generate(&env);

        env.mock_all_auths();
        client.initialize(&admin);

        let auth_address = match case.auth_role {
            "admin" => Some(admin.clone()),
            "wrong_user" => Some(wrong_user.clone()),
            "attester" => Some(attester.clone()),
            _ => None,
        };

        if let Some(addr) = auth_address {
            env.mock_auths(&[soroban_sdk::testutils::MockAuth {
                address: &addr,
                invoke: &soroban_sdk::testutils::MockAuthInvoke {
                    contract: &client.address,
                    fn_name: "add_attester",
                    args: (attester.clone(),).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
        } else {
            env.mock_auths(&[]);
        }

        let result = client.try_add_attester(&attester);
        assert_eq!(
            result, case.expected_result,
            "Failed case '{}': expected {:?}, got {:?}",
            case.name, case.expected_result, result
        );

        if case.expected_result.is_ok() {
            assert!(client.is_attester(&attester));
        } else {
            assert!(!client.is_attester(&attester));
        }
    }
}

#[test]
fn test_remove_attester_auth_matrix() {
    struct TestCase {
        name: &'static str,
        auth_role: &'static str, // "admin", "wrong_user", "none", "attester"
        expected_result: Result<Result<(), soroban_sdk::ConversionError>, Result<Error, soroban_sdk::InvokeError>>,
    }

    let cases = std::vec![
        TestCase {
            name: "Right Caller (Admin)",
            auth_role: "admin",
            expected_result: Ok(Ok(())),
        },
        TestCase {
            name: "Wrong Caller (Wrong User)",
            auth_role: "wrong_user",
            expected_result: Err(Err(soroban_sdk::InvokeError::Abort)),
        },
        TestCase {
            name: "No Auth Provided",
            auth_role: "none",
            expected_result: Err(Err(soroban_sdk::InvokeError::Abort)),
        },
        TestCase {
            name: "Role Confusion (Attester)",
            auth_role: "attester",
            expected_result: Err(Err(soroban_sdk::InvokeError::Abort)),
        },
    ];

    for case in cases {
        let env = Env::default();
        let contract_id = env.register(AttesterRegistry, ());
        let client = AttesterRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let wrong_user = Address::generate(&env);
        let attester = Address::generate(&env);

        env.mock_all_auths();
        client.initialize(&admin);
        client.add_attester(&attester);
        assert!(client.is_attester(&attester));

        let auth_address = match case.auth_role {
            "admin" => Some(admin.clone()),
            "wrong_user" => Some(wrong_user.clone()),
            "attester" => Some(attester.clone()),
            _ => None,
        };

        if let Some(addr) = auth_address {
            env.mock_auths(&[soroban_sdk::testutils::MockAuth {
                address: &addr,
                invoke: &soroban_sdk::testutils::MockAuthInvoke {
                    contract: &client.address,
                    fn_name: "remove_attester",
                    args: (attester.clone(),).into_val(&env),
                    sub_invokes: &[],
                },
            }]);
        } else {
            env.mock_auths(&[]);
        }

        let result = client.try_remove_attester(&attester);
        assert_eq!(
            result, case.expected_result,
            "Failed case '{}': expected {:?}, got {:?}",
            case.name, case.expected_result, result
        );

        if case.expected_result.is_ok() {
            assert!(!client.is_attester(&attester));
        } else {
            assert!(client.is_attester(&attester));
        }
    }
}

