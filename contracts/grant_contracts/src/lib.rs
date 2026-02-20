#![no_std]
use soroban_sdk::{contract, contractimpl, vec, Env, String, Vec};

#[contract]
pub struct Contract;

// This is a sample contract. Replace this placeholder with your own contract logic.
// A corresponding test example is available in `test.rs`.
//
// For comprehensive examples, visit <https://github.com/stellar/soroban-examples>.
// The repository includes use cases for the Stellar ecosystem, such as data storage on
// the blockchain, token swaps, liquidity pools, and more.
//
// Refer to the official documentation:
// <https://developers.stellar.org/docs/build/smart-contracts/overview>.
#[contractimpl]
impl Contract {
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "Hello"), to]
    }
}

mod test;

// Grant math utilities used by tests and (optionally) the contract.
pub mod grant {
    /// Compute the claimable balance for a linear vesting grant.
    ///
    /// - `total`: total amount granted (u128)
    /// - `start`: grant start timestamp (seconds, u64)
    /// - `now`: current timestamp (seconds, u64)
    /// - `duration`: grant duration (seconds, u64)
    ///
    /// Returns the amount (u128) claimable at `now` (clamped 0..=total).
    pub fn compute_claimable_balance(total: u128, start: u64, now: u64, duration: u64) -> u128 {
        if duration == 0 {
            return if now >= start { total } else { 0 };
        }
        if now <= start {
            return 0;
        }
        let elapsed = now.saturating_sub(start);
        if elapsed >= duration {
            return total;
        }

        // Use decomposition to reduce risk of intermediate overflow:
        // total * elapsed / duration == (total / duration) * elapsed + (total % duration) * elapsed / duration
        let dur = duration as u128;
        let el = elapsed as u128;
        let whole = total / dur;
        let rem = total % dur;

        // whole * el shouldn't overflow in realistic token amounts, but use checked_mul with fallback.
        let part1 = match whole.checked_mul(el) {
            Some(v) => v,
            None => {
                // fallback: perform (whole / dur) * (el * dur) approximated by dividing early
                // This branch is extremely unlikely; clamp to total as safe fallback.
                return total;
            }
        };
        let part2 = match rem.checked_mul(el) {
            Some(v) => v / dur,
            None => {
                return total;
            }
        };
        part1 + part2
    }
}
