// ============================================================================
// CONTRACT: MULTI-SIG WALLET
// ============================================================================

extern crate alloc;

use stylus_sdk::{
    prelude::*,
};
use alloy_primitives::{U256, Address};
use alloy_sol_types::sol;
use alloc::vec::Vec;

sol_storage! {
    #[entrypoint]
    pub struct MultiSigWallet {
        address[] owners;
        mapping(address => bool) is_owner;
        uint256 required_confirmations;
        uint256 transaction_count;
        mapping(uint256 => Transaction) transactions;
        mapping(uint256 => mapping(address => bool)) confirmations;
    }

    pub struct Transaction {
        address to;
        uint256 value;
        bytes data;
        bool executed;
        uint256 confirmation_count;
    }
}

#[public]
impl MultiSigWallet {
    /// Initialize wallet with owners and required confirmations
    pub fn new(&mut self, owners: Vec<Address>, required: U256) -> Result<(), Vec<u8>> {
        if owners.is_empty() {
            return Err("Owners required".as_bytes().to_vec());
        }
        if required == U256::from(0) || required > U256::from(owners.len()) {
            return Err("Invalid required confirmations".as_bytes().to_vec());
        }
        
        for owner in &owners {
            if *owner == Address::ZERO {
                return Err("Invalid owner address".as_bytes().to_vec());
            }
            if self.is_owner.get(*owner) {
                return Err("Duplicate owner".as_bytes().to_vec());
            }
            
            self.owners.push(*owner);
            self.is_owner.setter(*owner).set(true);
        }
        
        self.required_confirmations.set(required);
        self.transaction_count.set(U256::from(0));
        
        Ok(())
    }

    /// Submit a transaction
    pub fn submit_transaction(&mut self, to: Address, value: U256, data: Vec<u8>) -> Result<U256, Vec<u8>> {
        let sender = self.vm().msg_sender();
        
        if !self.is_owner.get(sender) {
            return Err("Not an owner".as_bytes().to_vec());
        }
        
        let tx_id = self.transaction_count.get();
        
        let mut transaction = self.transactions.setter(tx_id);
        transaction.to.set(to);
        transaction.value.set(value);
        transaction.data.set_bytes(&data);
        transaction.executed.set(false);
        transaction.confirmation_count.set(U256::from(0));
        
        self.transaction_count.set(tx_id + U256::from(1));
        
        log(self.vm(), TransactionSubmitted {
            transaction_id: tx_id,
            owner: sender,
            to,
            value,
        });
        
        Ok(tx_id)
    }

    /// Confirm a transaction
    pub fn confirm_transaction(&mut self, tx_id: U256) -> Result<(), Vec<u8>> {
        let sender = self.vm().msg_sender();
        
        if !self.is_owner.get(sender) {
            return Err("Not an owner".as_bytes().to_vec());
        }
        
        let transaction = self.transactions.get(tx_id);
        if transaction.executed.get() {
            return Err("Transaction already executed".as_bytes().to_vec());
        }
        if self.confirmations.get(tx_id).get(sender) {
            return Err("Transaction already confirmed".as_bytes().to_vec());
        }
        
        self.confirmations.setter(tx_id).setter(sender).set(true);
        
        // Update confirmation count
        let current_count = transaction.confirmation_count.get();
        self.transactions.setter(tx_id).confirmation_count.set(current_count + U256::from(1));
        
        log(self.vm(), TransactionConfirmed {
            transaction_id: tx_id,
            owner: sender,
        });
        
        Ok(())
    }

    /// Execute a transaction
    pub fn execute_transaction(&mut self, tx_id: U256) -> Result<(), Vec<u8>> {
        let transaction = self.transactions.get(tx_id);
        
        if transaction.executed.get() {
            return Err("Transaction already executed".as_bytes().to_vec());
        }
        if transaction.confirmation_count.get() < self.required_confirmations.get() {
            return Err("Not enough confirmations".as_bytes().to_vec());
        }
        
        self.transactions.setter(tx_id).executed.set(true);
        
        // Execute the transaction (simplified - in real implementation would use call)
        log(self.vm(), TransactionExecuted {
            transaction_id: tx_id,
        });
        
        Ok(())
    }

    /// Get transaction details
    pub fn get_transaction(&self, tx_id: U256) -> (Address, U256, Vec<u8>, bool, U256) {
        let transaction = self.transactions.get(tx_id);
        (
            transaction.to.get(),
            transaction.value.get(),
            transaction.data.get_bytes(),
            transaction.executed.get(),
            transaction.confirmation_count.get(),
        )
    }

    /// Check if transaction is confirmed by owner
    pub fn is_confirmed(&self, tx_id: U256, owner: Address) -> bool {
        self.confirmations.get(tx_id).get(owner)
    }

    /// Get number of confirmations for transaction
    pub fn get_confirmation_count(&self, tx_id: U256) -> U256 {
        self.transactions.get(tx_id).confirmation_count.get()
    }

    /// Get required confirmations
    pub fn get_required_confirmations(&self) -> U256 {
        self.required_confirmations.get()
    }

    /// Get total transaction count
    pub fn get_transaction_count(&self) -> U256 {
        self.transaction_count.get()
    }

    /// Get owner count
    pub fn get_owner_count(&self) -> U256 {
        U256::from(self.owners.len())
    }

    /// Check if address is owner
    pub fn is_owner(&self, addr: Address) -> bool {
        self.is_owner.get(addr)
    }
}

sol! {
    event TransactionSubmitted(uint256 indexed transaction_id, address indexed owner, address indexed to, uint256 value);
    event TransactionConfirmed(uint256 indexed transaction_id, address indexed owner);
    event TransactionExecuted(uint256 indexed transaction_id);
}