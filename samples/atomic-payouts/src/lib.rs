#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec};

/// This sample's guard, not a Crossmint or Stellar product limit.
pub const MAX_PAYMENTS: u32 = 32;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub amount: i128,
    pub to: Address,
}

#[contract]
pub struct BatchPayments;

#[contractimpl]
impl BatchPayments {
    /// Transfers from one wallet to every recipient in one atomic invocation.
    /// Amounts are integer token base units. The helper never holds the funds.
    pub fn pay(env: Env, token: Address, from: Address, payments: Vec<Payment>) {
        from.require_auth();
        assert!(!payments.is_empty(), "empty batch");
        assert!(payments.len() <= MAX_PAYMENTS, "batch too large");

        let client = token::Client::new(&env, &token);
        for payment in payments.iter() {
            assert!(payment.amount > 0, "amount must be positive");
            // Do not catch failures: a failed transfer must revert the whole batch.
            client.transfer(&from, &payment.to, &payment.amount);
        }
    }
}

#[cfg(test)]
mod test;
