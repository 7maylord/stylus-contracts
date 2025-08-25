//! Example Stylus contract demonstrating ReentrancyGuard usage
//! 
//! This contract shows how to use the ReentrancyGuard to protect against
//! reentrancy attacks in a simple vault contract.

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::{Address, U256};
use stylus_sdk::{
    alloy_sol_types::sol,
    call::Call,
    contract,
    msg,
    prelude::*,
    storage::{StorageAddress, StorageU256, StorageMap},
};

// Import our reentrancy guard
mod reentrancy;
use reentrancy::{ReentrancyGuard, ReentrancyGuarded, ReentrancyError};

// Solidity interface definitions
sol! {
    event Deposit(address indexed user, uint256 amount);
    event Withdrawal(address indexed user, uint256 amount);
    
    error InsufficientBalance();
    error WithdrawalFailed();
}

/// Contract errors
#[derive(SolidityError)]
pub enum VaultError {
    #[solidity(error = "InsufficientBalance()")]
    InsufficientBalance,
    
    #[solidity(error = "WithdrawalFailed()")]
    WithdrawalFailed,
    
    #[solidity(error = "ReentrancyGuardReentrantCall()")]
    ReentrantCall,
}

impl From<ReentrancyError> for VaultError {
    fn from(err: ReentrancyError) -> Self {
        match err {
            ReentrancyError::ReentrantCall => VaultError::ReentrantCall,
        }
    }
}

/// Main contract storage
#[entrypoint]
#[storage]
pub struct VaultContract {
    /// ReentrancyGuard for protection
    guard: ReentrancyGuard,
    /// User balances
    balances: StorageMap<Address, StorageU256>,
    /// Total contract balance
    total_balance: StorageU256,
}

impl ReentrancyGuarded for VaultContract {
    fn reentrancy_guard(&mut self) -> &mut ReentrancyGuard {
        &mut self.guard
    }
}

/// Public interface implementation
#[public]
impl VaultContract {
    /// Constructor - initializes the ReentrancyGuard
    #[constructor]
    pub fn constructor(&mut self) {
        self.guard.init();
        self.total_balance.set(U256::ZERO);
    }

    /// Deposit ETH into the vault
    /// 
    /// This function is protected against reentrancy attacks.
    #[payable]
    pub fn deposit(&mut self) -> Result<(), VaultError> {
        let caller = msg::sender();
        let amount = msg::value();
        
        // Use reentrancy protection
        self.with_non_reentrant(|contract| {
            let current_balance = contract.balances.getter(caller).get();
            let new_balance = current_balance + amount;
            
            contract.balances.setter(caller).set(new_balance);
            contract.total_balance.set(contract.total_balance.get() + amount);
            
            // Emit deposit event
            evm::log(Deposit {
                user: caller,
                amount,
            });
        })
    }

    /// Withdraw ETH from the vault (VULNERABLE VERSION - for demonstration)
    /// 
    /// This version is intentionally vulnerable to show what happens without protection.
    /// DO NOT USE THIS IN PRODUCTION!
    pub fn withdraw_vulnerable(&mut self, amount: U256) -> Result<(), VaultError> {
        let caller = msg::sender();
        let balance = self.balances.getter(caller).get();
        
        if balance < amount {
            return Err(VaultError::InsufficientBalance);
        }
        
        // Update balance AFTER external call - VULNERABLE!
        let call_result = Call::new_in(self)
            .value(amount)
            .call(caller, &[]);
        
        if call_result.is_err() {
            return Err(VaultError::WithdrawalFailed);
        }
        
        // State changes after external call - vulnerable to reentrancy
        self.balances.setter(caller).set(balance - amount);
        self.total_balance.set(self.total_balance.get() - amount);
        
        evm::log(Withdrawal {
            user: caller,
            amount,
        });
        
        Ok(())
    }

    /// Withdraw ETH from the vault (SAFE VERSION with ReentrancyGuard)
    /// 
    /// This version uses the ReentrancyGuard to prevent reentrancy attacks.
    pub fn withdraw_safe(&mut self, amount: U256) -> Result<(), VaultError> {
        let caller = msg::sender();
        
        // Use reentrancy protection
        self.with_non_reentrant(|contract| {
            let balance = contract.balances.getter(caller).get();
            
            if balance < amount {
                return Err(VaultError::InsufficientBalance);
            }
            
            // Update state BEFORE external call - follows CEI pattern
            contract.balances.setter(caller).set(balance - amount);
            contract.total_balance.set(contract.total_balance.get() - amount);
            
            // External call after state changes
            let call_result = Call::new_in(contract)
                .value(amount)
                .call(caller, &[]);
            
            if call_result.is_err() {
                // Revert state changes if call fails
                contract.balances.setter(caller).set(balance);
                contract.total_balance.set(contract.total_balance.get() + amount);
                return Err(VaultError::WithdrawalFailed);
            }
            
            evm::log(Withdrawal {
                user: caller,
                amount,
            });
            
            Ok(())
        })?
    }

    /// Get the balance of a user
    pub fn get_balance(&self, user: Address) -> U256 {
        self.balances.getter(user).get()
    }

    /// Get the total contract balance
    pub fn get_total_balance(&self) -> U256 {
        self.total_balance.get()
    }

    /// Check if the contract is currently in a reentrant call
    /// 
    /// This can be useful for debugging or conditional logic.
    pub fn is_reentrancy_guard_entered(&self) -> bool {
        self.guard.reentrancy_guard_entered()
    }

    /// Emergency function that demonstrates manual reentrancy protection
    /// 
    /// This shows how to use the guard manually without the trait helper.
    pub fn emergency_withdraw(&mut self, amount: U256) -> Result<(), VaultError> {
        let caller = msg::sender();
        
        // Manual reentrancy protection
        self.guard.non_reentrant_before()?;
        
        let balance = self.balances.getter(caller).get();
        
        if balance < amount {
            self.guard.non_reentrant_after();
            return Err(VaultError::InsufficientBalance);
        }
        
        // Update state before external call
        self.balances.setter(caller).set(balance - amount);
        self.total_balance.set(self.total_balance.get() - amount);
        
        // External call
        let call_result = Call::new_in(self)
            .value(amount)
            .call(caller, &[]);
        
        // Always clean up reentrancy guard
        self.guard.non_reentrant_after();
        
        if call_result.is_err() {
            // Revert state if call failed
            self.balances.setter(caller).set(balance);
            self.total_balance.set(self.total_balance.get() + amount);
            return Err(VaultError::WithdrawalFailed);
        }
        
        evm::log(Withdrawal {
            user: caller,
            amount,
        });
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit() {
        let mut contract = VaultContract::default();
        contract.constructor();
        
        // Simulate a deposit (in real tests, you'd use stylus test helpers)
        let user = Address::ZERO;
        let amount = U256::from(100);
        
        // In actual tests, you'd set up the message context properly
        assert_eq!(contract.get_balance(user), U256::ZERO);
    }

    #[test]
    fn test_reentrancy_protection() {
        let mut contract = VaultContract::default();
        contract.constructor();
        
        // Verify guard is initialized
        assert!(!contract.is_reentrancy_guard_entered());
        
        // Test manual guard usage
        assert!(contract.guard.non_reentrant_before().is_ok());
        assert!(contract.is_reentrancy_guard_entered());
        
        // Should fail on second call
        assert!(contract.guard.non_reentrant_before().is_err());
        
        // Clean up
        contract.guard.non_reentrant_after();
        assert!(!contract.is_reentrancy_guard_entered());
    }
}