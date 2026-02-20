#![no_std]
use soroban_sdk::{
    contract, contractimpl, map, symbol_short, Address, Bytes, Env, Map, String, Symbol, Val,
};

// Contract for managing milestone-based grant unlocking
// Grants can be unlocked via admin approval of specific milestones

#[derive(Clone)]
pub struct Grant {
    pub admin: Address,
    pub grantee: Address,
    pub total_amount: i128,
    pub released_amount: i128,
}

#[derive(Clone)]
pub struct Milestone {
    pub amount: i128,
    pub status: u32, // 0 = Pending, 1 = Approved, 2 = Released
    pub description: String,
}

#[contract]
pub struct GrantContract;

#[contractimpl]
impl GrantContract {
    /// Create a new grant with an admin and grantee
    /// Only called once per grant ID
    ///
    /// Args:
    /// - grant_id: Unique identifier for the grant
    /// - admin: Admin address who can approve milestones
    /// - grantee: Address receiving the grant funds
    /// - total_amount: Total grant amount in stroops
    pub fn create_grant(
        env: Env,
        grant_id: Symbol,
        admin: Address,
        grantee: Address,
        total_amount: i128,
    ) -> Result<Symbol, String> {
        // Verify admin address
        admin.require_auth();

        // Check if grant already exists
        if env.storage().persistent().has(&grant_id) {
            return Err(String::from_str(&env, "Grant already exists"));
        }

        // Create and store grant
        let grant = Grant {
            admin: admin.clone(),
            grantee: grantee.clone(),
            total_amount,
            released_amount: 0,
        };

        env.storage()
            .persistent()
            .set(&grant_id, &grant);

        env.events()
            .publish((symbol_short!("grant"), symbol_short!("created")), grant_id.clone());

        Ok(grant_id)
    }

    /// Add a new milestone to a grant
    /// Only the admin can call this
    ///
    /// Args:
    /// - grant_id: ID of the grant
    /// - milestone_id: Unique identifier for the milestone
    /// - amount: Amount to be released when milestone is approved
    /// - description: Description of the milestone
    pub fn add_milestone(
        env: Env,
        grant_id: Symbol,
        milestone_id: Symbol,
        amount: i128,
        description: String,
    ) -> Result<Symbol, String> {
        // Get grant to verify admin
        let grant: Grant = env
            .storage()
            .persistent()
            .get(&grant_id)
            .ok_or(String::from_str(&env, "Grant not found"))?;

        grant.admin.require_auth();

        // Create milestone key
        let mut milestone_key_string = String::from_str(&env, "milestone:");
        milestone_key_string.append(&grant_id.to_string());
        milestone_key_string.append(&String::from_str(&env, ":"));
        milestone_key_string.append(&milestone_id.to_string());
        
        let milestone_key = Symbol::new(&env, &milestone_key_string);

        // Check if milestone already exists
        if env.storage().persistent().has(&milestone_key) {
            return Err(String::from_str(&env, "Milestone already exists"));
        }

        // Create and store milestone
        let milestone = Milestone {
            amount,
            status: 0, // Pending
            description,
        };

        env.storage()
            .persistent()
            .set(&milestone_key, &milestone);

        env.events().publish(
            (symbol_short!("milestone"), symbol_short!("created")),
            (grant_id.clone(), milestone_id.clone()),
        );

        Ok(milestone_id)
    }

    /// Get milestone details
    pub fn get_milestone(
        env: Env,
        grant_id: Symbol,
        milestone_id: Symbol,
    ) -> Result<(i128, u32, String), String> {
        // Create milestone key
        let mut milestone_key_string = String::from_str(&env, "milestone:");
        milestone_key_string.append(&grant_id.to_string());
        milestone_key_string.append(&String::from_str(&env, ":"));
        milestone_key_string.append(&milestone_id.to_string());
        
        let milestone_key = Symbol::new(&env, &milestone_key_string);

        let milestone: Milestone = env
            .storage()
            .persistent()
            .get(&milestone_key)
            .ok_or(String::from_str(&env, "Milestone not found"))?;

        Ok((milestone.amount, milestone.status, milestone.description))
    }

    /// Approve a milestone and release funds immediately to grantee
    /// Only admin can call this
    ///
    /// Args:
    /// - grant_id: ID of the grant
    /// - milestone_id: ID of the milestone to approve
    pub fn approve_milestone(
        env: Env,
        grant_id: Symbol,
        milestone_id: Symbol,
    ) -> Result<i128, String> {
        // Get grant
        let mut grant: Grant = env
            .storage()
            .persistent()
            .get(&grant_id)
            .ok_or(String::from_str(&env, "Grant not found"))?;

        grant.admin.require_auth();

        // Get milestone
        let mut milestone_key_string = String::from_str(&env, "milestone:");
        milestone_key_string.append(&grant_id.to_string());
        milestone_key_string.append(&String::from_str(&env, ":"));
        milestone_key_string.append(&milestone_id.to_string());
        
        let milestone_key = Symbol::new(&env, &milestone_key_string);

        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&milestone_key)
            .ok_or(String::from_str(&env, "Milestone not found"))?;

        // Check if already released
        if milestone.status == 2 {
            return Err(String::from_str(&env, "Milestone already released"));
        }

        // Check if total released + this amount exceeds total grant
        if grant.released_amount + milestone.amount > grant.total_amount {
            return Err(String::from_str(&env, "Exceeds total grant amount"));
        }

        // Update milestone status to Released
        milestone.status = 2;
        env.storage()
            .persistent()
            .set(&milestone_key, &milestone);

        // Update grant released amount
        grant.released_amount += milestone.amount;
        env.storage()
            .persistent()
            .set(&grant_id, &grant);

        // Emit event with released amount
        env.events().publish(
            (symbol_short!("milestone"), symbol_short!("released")),
            (grant_id.clone(), milestone_id.clone(), milestone.amount),
        );

        Ok(milestone.amount)
    }

    /// Get grant details
    pub fn get_grant(
        env: Env,
        grant_id: Symbol,
    ) -> Result<(Address, Address, i128, i128), String> {
        let grant: Grant = env
            .storage()
            .persistent()
            .get(&grant_id)
            .ok_or(String::from_str(&env, "Grant not found"))?;

        Ok((
            grant.admin,
            grant.grantee,
            grant.total_amount,
            grant.released_amount,
        ))
    }

    /// Get total released amount for a grant
    pub fn get_released_amount(env: Env, grant_id: Symbol) -> Result<i128, String> {
        let grant: Grant = env
            .storage()
            .persistent()
            .get(&grant_id)
            .ok_or(String::from_str(&env, "Grant not found"))?;

        Ok(grant.released_amount)
    }

    /// Get remaining amount available in a grant
    pub fn get_remaining_amount(env: Env, grant_id: Symbol) -> Result<i128, String> {
        let grant: Grant = env
            .storage()
            .persistent()
            .get(&grant_id)
            .ok_or(String::from_str(&env, "Grant not found"))?;

        Ok(grant.total_amount - grant.released_amount)
    }
}

mod test;
