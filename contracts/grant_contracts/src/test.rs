#![cfg(test)]

use super::*;
use soroban_sdk::{Address, Env, Symbol, String};

#[test]
fn test_create_grant() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let grantee = Address::generate(&env);

    let contract_id = env.register(GrantContract, ());
    let client = GrantContractClient::new(&env, &contract_id);

    // Create a grant
    let grant_id = Symbol::new(&env, "grant_1");
    let result = client.create_grant(
        &grant_id,
        &admin,
        &grantee,
        &1_000_000,
    );

    assert!(result.is_ok());
    let returned_id = result.unwrap();
    assert_eq!(returned_id, grant_id);

    // Verify grant details
    let grant_info = client.get_grant(&grant_id).unwrap();
    assert_eq!(grant_info.0, admin);
    assert_eq!(grant_info.1, grantee);
    assert_eq!(grant_info.2, 1_000_000); // total
    assert_eq!(grant_info.3, 0); // released
}

#[test]
fn test_create_duplicate_grant() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let grantee = Address::generate(&env);

    let contract_id = env.register(GrantContract, ());
    let client = GrantContractClient::new(&env, &contract_id);

    let grant_id = Symbol::new(&env, "grant_dup");
    
    // First creation should succeed
    let result1 = client.create_grant(&grant_id, &admin, &grantee, &1_000_000);
    assert!(result1.is_ok());

    // Second creation with same ID should fail
    let result2 = client.create_grant(&grant_id, &admin, &grantee, &1_000_000);
    assert!(result2.is_err());
}

#[test]
fn test_add_milestone() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let grantee = Address::generate(&env);

    let contract_id = env.register(GrantContract, ());
    let client = GrantContractClient::new(&env, &contract_id);

    // Create a grant
    let grant_id = Symbol::new(&env, "grant_mvp");
    client.create_grant(&grant_id, &admin, &grantee, &1_000_000).unwrap();

    // Add a milestone
    let milestone_id = Symbol::new(&env, "mvp_delivered");
    let description = String::from_str(&env, "MVP Delivered to Beta Testers");
    
    let result = client.add_milestone(
        &grant_id,
        &milestone_id,
        &500_000,
        &description,
    );

    assert!(result.is_ok());

    // Verify milestone details
    let milestone_info = client.get_milestone(&grant_id, &milestone_id).unwrap();
    assert_eq!(milestone_info.0, 500_000); // amount
    assert_eq!(milestone_info.1, 0); // status = Pending
    assert_eq!(milestone_info.2, description);
}

#[test]
fn test_approve_milestone() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let grantee = Address::generate(&env);

    let contract_id = env.register(GrantContract, ());
    let client = GrantContractClient::new(&env, &contract_id);

    // Create a grant
    let grant_id = Symbol::new(&env, "grant_test");
    client.create_grant(&grant_id, &admin, &grantee, &1_000_000).unwrap();

    // Add a milestone
    let milestone_id = Symbol::new(&env, "milestone_1");
    client.add_milestone(
        &grant_id,
        &milestone_id,
        &300_000,
        &String::from_str(&env, "First Milestone"),
    ).unwrap();

    // Approve the milestone
    let released_amount = client.approve_milestone(&grant_id, &milestone_id).unwrap();
    assert_eq!(released_amount, 300_000);

    // Verify milestone status changed to Released (2)
    let milestone_info = client.get_milestone(&grant_id, &milestone_id).unwrap();
    assert_eq!(milestone_info.1, 2); // status = Released

    // Verify grant released amount updated
    let grant_info = client.get_grant(&grant_id).unwrap();
    assert_eq!(grant_info.3, 300_000); // released amount
}

#[test]
fn test_multiple_milestones() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let grantee = Address::generate(&env);

    let contract_id = env.register(GrantContract, ());
    let client = GrantContractClient::new(&env, &contract_id);

    // Create a grant
    let grant_id = Symbol::new(&env, "grant_multi");
    client.create_grant(&grant_id, &admin, &grantee, &1_000_000).unwrap();

    // Add multiple milestones
    let milestone_1 = Symbol::new(&env, "m1");
    let milestone_2 = Symbol::new(&env, "m2");
    let milestone_3 = Symbol::new(&env, "m3");

    client.add_milestone(&grant_id, &milestone_1, &250_000, &String::from_str(&env, "Phase 1")).unwrap();
    client.add_milestone(&grant_id, &milestone_2, &350_000, &String::from_str(&env, "Phase 2")).unwrap();
    client.add_milestone(&grant_id, &milestone_3, &400_000, &String::from_str(&env, "Phase 3")).unwrap();

    // Approve first milestone
    client.approve_milestone(&grant_id, &milestone_1).unwrap();
    let grant_info = client.get_grant(&grant_id).unwrap();
    assert_eq!(grant_info.3, 250_000);

    // Approve second milestone
    client.approve_milestone(&grant_id, &milestone_2).unwrap();
    let grant_info = client.get_grant(&grant_id).unwrap();
    assert_eq!(grant_info.3, 600_000);

    // Approve third milestone
    client.approve_milestone(&grant_id, &milestone_3).unwrap();
    let grant_info = client.get_grant(&grant_id).unwrap();
    assert_eq!(grant_info.3, 1_000_000);
}

#[test]
fn test_double_release_prevention() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let grantee = Address::generate(&env);

    let contract_id = env.register(GrantContract, ());
    let client = GrantContractClient::new(&env, &contract_id);

    // Create a grant and milestone
    let grant_id = Symbol::new(&env, "grant_double");
    client.create_grant(&grant_id, &admin, &grantee, &1_000_000).unwrap();

    let milestone_id = Symbol::new(&env, "milestone_double");
    client.add_milestone(
        &grant_id,
        &milestone_id,
        &500_000,
        &String::from_str(&env, "Test"),
    ).unwrap();

    // Approve once
    client.approve_milestone(&grant_id, &milestone_id).unwrap();

    // Try to approve again - should fail
    let result = client.approve_milestone(&grant_id, &milestone_id);
    assert!(result.is_err());
}

#[test]
fn test_get_remaining_amount() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let grantee = Address::generate(&env);

    let contract_id = env.register(GrantContract, ());
    let client = GrantContractClient::new(&env, &contract_id);

    // Create a grant
    let grant_id = Symbol::new(&env, "grant_remaining");
    client.create_grant(&grant_id, &admin, &grantee, &1_000_000).unwrap();

    // Check remaining amount before any releases
    let remaining = client.get_remaining_amount(&grant_id).unwrap();
    assert_eq!(remaining, 1_000_000);

    // Add and approve a milestone
    let milestone_id = Symbol::new(&env, "m1");
    client.add_milestone(&grant_id, &milestone_id, &400_000, &String::from_str(&env, "Phase 1")).unwrap();
    client.approve_milestone(&grant_id, &milestone_id).unwrap();

    // Check remaining amount after release
    let remaining = client.get_remaining_amount(&grant_id).unwrap();
    assert_eq!(remaining, 600_000);
}

#[test]
fn test_exceed_total_grant_amount() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let grantee = Address::generate(&env);

    let contract_id = env.register(GrantContract, ());
    let client = GrantContractClient::new(&env, &contract_id);

    // Create a grant with 1M total
    let grant_id = Symbol::new(&env, "grant_exceed");
    client.create_grant(&grant_id, &admin, &grantee, &1_000_000).unwrap();

    // Add milestone for 600K
    let milestone_1 = Symbol::new(&env, "m1");
    client.add_milestone(&grant_id, &milestone_1, &600_000, &String::from_str(&env, "Phase 1")).unwrap();
    client.approve_milestone(&grant_id, &milestone_1).unwrap();

    // Add milestone for 500K (would exceed total)
    let milestone_2 = Symbol::new(&env, "m2");
    client.add_milestone(&grant_id, &milestone_2, &500_000, &String::from_str(&env, "Phase 2")).unwrap();

    // Trying to approve should fail
    let result = client.approve_milestone(&grant_id, &milestone_2);
    assert!(result.is_err());
}
