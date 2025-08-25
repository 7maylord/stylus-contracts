//! ReentrancyGuard module for Stylus smart contracts
//! 
//! This module provides reentrancy protection inspired by OpenZeppelin's ReentrancyGuard.sol.
//! It prevents reentrant calls to protected functions in Stylus-based smart contracts.

use alloy_primitives::U256;
use stylus_sdk::{
    prelude::*,
    storage::{StorageU256},
};

/// Error types for ReentrancyGuard
#[derive(SolidityError)]
pub enum ReentrancyError {
    /// Thrown when a reentrant call is detected
    #[solidity(error = "ReentrancyGuardReentrantCall()")]
    ReentrantCall,
}

/// ReentrancyGuard constants following OpenZeppelin's pattern
pub const NOT_ENTERED: U256 = U256::from_limbs([1, 0, 0, 0]);
pub const ENTERED: U256 = U256::from_limbs([2, 0, 0, 0]);

/// Storage structure for ReentrancyGuard
/// 
/// This struct holds the reentrancy status and should be included
/// in contracts that need reentrancy protection.
#[storage]
pub struct ReentrancyGuard {
    /// Current reentrancy status (NOT_ENTERED = 1, ENTERED = 2)
    status: StorageU256,
}

impl ReentrancyGuard {
    /// Initialize the ReentrancyGuard
    /// 
    /// Sets the initial status to NOT_ENTERED.
    /// This should be called in the contract's constructor.
    pub fn init(&mut self) {
        self.status.set(NOT_ENTERED);
    }

    /// Check if the contract is currently in a reentrant call
    /// 
    /// Returns true if there is a nonReentrant function in the call stack.
    pub fn reentrancy_guard_entered(&self) -> bool {
        self.status.get() == ENTERED
    }

    /// Internal function called before executing a non-reentrant function
    /// 
    /// Checks if the contract is already entered and throws an error if so,
    /// otherwise sets the status to ENTERED.
    /// 
    /// # Errors
    /// 
    /// Returns `ReentrancyError::ReentrantCall` if a reentrant call is detected.
    pub fn non_reentrant_before(&mut self) -> Result<(), ReentrancyError> {
        // On the first call to nonReentrant, status will be NOT_ENTERED
        if self.status.get() == ENTERED {
            return Err(ReentrancyError::ReentrantCall);
        }

        // Any calls to nonReentrant after this point will fail
        self.status.set(ENTERED);
        Ok(())
    }

    /// Internal function called after executing a non-reentrant function
    /// 
    /// Resets the status back to NOT_ENTERED.
    pub fn non_reentrant_after(&mut self) {
        // By storing the original value once again, a refund is triggered
        self.status.set(NOT_ENTERED);
    }

    /// Convenience method that wraps a closure with reentrancy protection
    /// 
    /// This method automatically handles the before/after logic for reentrancy protection.
    /// 
    /// # Arguments
    /// 
    /// * `f` - The closure to execute with reentrancy protection
    /// 
    /// # Errors
    /// 
    /// Returns `ReentrancyError::ReentrantCall` if a reentrant call is detected,
    /// or any error returned by the closure.
    pub fn non_reentrant<F, T, E>(&mut self, f: F) -> Result<T, ReentrancyError>
    where
        F: FnOnce() -> Result<T, E>,
        ReentrancyError: From<E>,
    {
        self.non_reentrant_before()?;
        let result = f().map_err(ReentrancyError::from);
        self.non_reentrant_after();
        result
    }
}

/// Trait for contracts that use ReentrancyGuard
/// 
/// This trait provides a convenient interface for contracts to use reentrancy protection.
pub trait ReentrancyGuarded {
    /// Get a mutable reference to the ReentrancyGuard
    fn reentrancy_guard(&mut self) -> &mut ReentrancyGuard;

    /// Execute a function with reentrancy protection
    /// 
    /// # Arguments
    /// 
    /// * `f` - The closure to execute with reentrancy protection
    /// 
    /// # Errors
    /// 
    /// Returns `ReentrancyError::ReentrantCall` if a reentrant call is detected.
    fn with_non_reentrant<F, T>(&mut self, f: F) -> Result<T, ReentrancyError>
    where
        F: FnOnce(&mut Self) -> T,
    {
        self.reentrancy_guard().non_reentrant_before()?;
        let result = f(self);
        self.reentrancy_guard().non_reentrant_after();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stylus_sdk::storage::StorageType;

    #[storage]
    struct TestContract {
        guard: ReentrancyGuard,
        counter: StorageU256,
    }

    impl ReentrancyGuarded for TestContract {
        fn reentrancy_guard(&mut self) -> &mut ReentrancyGuard {
            &mut self.guard
        }
    }

    #[test]
    fn test_initial_state() {
        let mut contract = TestContract::default();
        contract.guard.init();
        assert!(!contract.guard.reentrancy_guard_entered());
    }

    #[test]
    fn test_non_reentrant_protection() {
        let mut contract = TestContract::default();
        contract.guard.init();
        
        // First call should succeed
        let result = contract.guard.non_reentrant_before();
        assert!(result.is_ok());
        assert!(contract.guard.reentrancy_guard_entered());
        
        // Second call should fail
        let result = contract.guard.non_reentrant_before();
        assert!(matches!(result, Err(ReentrancyError::ReentrantCall)));
        
        // After cleanup, should work again
        contract.guard.non_reentrant_after();
        assert!(!contract.guard.reentrancy_guard_entered());
        
        let result = contract.guard.non_reentrant_before();
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_trait() {
        let mut contract = TestContract::default();
        contract.guard.init();
        
        let result = contract.with_non_reentrant(|c| {
            c.counter.set(U256::from(42));
            c.counter.get()
        });
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), U256::from(42));
        assert!(!contract.guard.reentrancy_guard_entered());
    }
}