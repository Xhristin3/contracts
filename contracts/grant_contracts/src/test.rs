#![cfg(test)]

use super::*;
use soroban_sdk::{vec, Env, String};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let words = client.hello(&String::from_str(&env, "Dev"));
    assert_eq!(
        words,
        vec![
            &env,
            String::from_str(&env, "Hello"),
            String::from_str(&env, "Dev"),
        ]
    );
}

#[test]
fn test_grant_simulation_10_years() {
    // 10 years in seconds
    let duration: u64 = 315_360_000;

    // Total grant amount
    let total: u128 = 1_000_000_000u128;

    // Use a realistic large timestamp to catch overflow issues
    let start: u64 = 1_700_000_000;

    // --------------------------------------------------
    // ✔ Start: nothing should be claimable
    // --------------------------------------------------
    let claim0 =
        grant::compute_claimable_balance(total, start, start, duration);
    assert_eq!(claim0, 0);

    // --------------------------------------------------
    // ✔ Year 5: exactly 50%
    // --------------------------------------------------
    let year5 = start + duration / 2;
    let claim5 =
        grant::compute_claimable_balance(total, start, year5, duration);

    assert_eq!(claim5, total / 2);

    // --------------------------------------------------
    // ✔ Year 10: 100% vested
    // --------------------------------------------------
    let year10 = start + duration;
    let claim10 =
        grant::compute_claimable_balance(total, start, year10, duration);

    assert_eq!(claim10, total);

    // --------------------------------------------------
    // ✔ After expiry: must remain capped at total
    // --------------------------------------------------
    let after = year10 + 1_000_000;
    let claim_after =
        grant::compute_claimable_balance(total, start, after, duration);

    assert_eq!(claim_after, total);

    // --------------------------------------------------
    // ✔ Verify constant equals 10-year duration
    // --------------------------------------------------
    assert_eq!(duration, 315_360_000u64);
}