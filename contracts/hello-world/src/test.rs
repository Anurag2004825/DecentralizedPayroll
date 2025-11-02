#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PayrollContract);
    let client = PayrollContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(&owner, &token);

    assert_eq!(client.get_owner(), owner);
}

#[test]
fn test_add_employee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PayrollContract);
    let client = PayrollContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let employee = Address::generate(&env);

    client.initialize(&owner, &token);

    let name = String::from_str(&env, "John Doe");
    client.add_employee(&owner, &employee, &100_000, &name);

    let emp_data = client.get_employee(&employee);
    assert_eq!(emp_data.salary, 100_000);
    assert_eq!(emp_data.name, name);
    assert_eq!(emp_data.active, true);
}

#[test]
fn test_update_salary() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PayrollContract);
    let client = PayrollContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let employee = Address::generate(&env);

    client.initialize(&owner, &token);

    let name = String::from_str(&env, "John Doe");
    client.add_employee(&owner, &employee, &100_000, &name);
    client.update_salary(&owner, &employee, &120_000);

    let emp_data = client.get_employee(&employee);
    assert_eq!(emp_data.salary, 120_000);
}

#[test]
fn test_deactivate_employee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PayrollContract);
    let client = PayrollContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let employee = Address::generate(&env);

    client.initialize(&owner, &token);

    let name = String::from_str(&env, "John Doe");
    client.add_employee(&owner, &employee, &100_000, &name);
    client.deactivate_employee(&owner, &employee);

    let emp_data = client.get_employee(&employee);
    assert_eq!(emp_data.active, false);
}

#[test]
#[should_panic(expected = "Only owner can perform this action")]
fn test_non_owner_cannot_add_employee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PayrollContract);
    let client = PayrollContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let token = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let employee = Address::generate(&env);

    client.initialize(&owner, &token);

    let name = String::from_str(&env, "John Doe");
    client.add_employee(&non_owner, &employee, &100_000, &name);
}