# 🛡️ ReentrancyGuard for Stylus

A secure, gas-efficient reentrancy protection implementation for Arbitrum Stylus smart contracts, inspired by [OpenZeppelin's `ReentrancyGuard.sol`](https://github.com/OpenZeppelin/openzeppelin-contracts/blob/master/contracts/utils/ReentrancyGuard.sol).

## 🎯 Overview

This library provides reentrancy protection for Rust-based smart contracts deployed on Arbitrum using Stylus. It prevents malicious contracts from re-entering your functions during external calls, protecting against one of the most common smart contract vulnerabilities.

## ⚡ Features

- **Gas Efficient**: Uses the same optimization as OpenZeppelin (uint256 instead of bool)
- **Type Safe**: Leverages Rust's type system for additional safety guarantees
- **Easy Integration**: Simple trait-based API for seamless integration
- **Multiple Usage Patterns**: Support for both manual and automatic protection
- **No External Dependencies**: Pure Stylus SDK implementation
- **Well Tested**: Comprehensive test suite included

## 🚀 Quick Start

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
reentrancy-guard-stylus = "0.1.0"
stylus-sdk = "0.9.0"
alloy-primitives = "0.8.20"
```

### Basic Usage

```rust
use reentrancy_guard_stylus::{ReentrancyGuard, ReentrancyGuarded, ReentrancyError};
use stylus_sdk::prelude::*;

#[entrypoint]
#[storage]
pub struct MyContract {
    guard: ReentrancyGuard,
    // ... other fields
}

impl ReentrancyGuarded for MyContract {
    fn reentrancy_guard(&mut self) -> &mut ReentrancyGuard {
        &mut self.guard
    }
}

#[public]
impl MyContract {
    #[constructor]
    pub fn constructor(&mut self) {
        self.guard.init();
    }

    pub fn protected_function(&mut self) -> Result<(), ReentrancyError> {
        self.with_non_reentrant(|contract| {
            // Your protected code here
            // This cannot be called recursively
        })
    }
}
```

## 📚 API Reference

### `ReentrancyGuard` Struct

The main storage component that tracks reentrancy status.

#### Methods

- `init()` - Initialize the guard (call in constructor)
- `non_reentrant_before()` - Check and set entered state
- `non_reentrant_after()` - Reset to not-entered state
- `reentrancy_guard_entered()` - Check if currently in reentrant call
- `non_reentrant<F, T, E>(f: F)` - Execute closure with protection

### `ReentrancyGuarded` Trait

Provides convenient helper methods for contracts using the guard.

#### Methods

- `reentrancy_guard()` - Get mutable reference to the guard
- `with_non_reentrant<F, T>(f: F)` - Execute closure with automatic protection

### Constants

- `NOT_ENTERED = 1` - Default state (not in a protected function)
- `ENTERED = 2` - Protected state (currently in a protected function)

## 🛠️ Usage Patterns

### 1. Automatic Protection (Recommended)

```rust
pub fn withdraw(&mut self, amount: U256) -> Result<(), VaultError> {
    self.with_non_reentrant(|contract| {
        // Check conditions
        let balance = contract.get_balance(msg::sender());
        if balance < amount {
            return Err(VaultError::InsufficientBalance);
        }
        
        // Update state BEFORE external call
        contract.set_balance(msg::sender(), balance - amount);
        
        // External call
        contract.transfer_to(msg::sender(), amount)?;
        
        Ok(())
    })?
}
```

### 2. Manual Protection

```rust
pub fn emergency_withdraw(&mut self, amount: U256) -> Result<(), VaultError> {
    // Manual guard management
    self.guard.non_reentrant_before()?;
    
    let result = self.do_withdrawal(amount);
    
    // Always clean up, even on error
    self.guard.non_reentrant_after();
    
    result
}
```

### 3. Checking Reentrancy Status

```rust
pub fn is_reentrant(&self) -> bool {
    self.guard.reentrancy_guard_entered()
}

pub fn conditional_logic(&mut self) {
    if msg::reentrant() {
        // Special handling for reentrant calls
        // (if reentrancy is enabled in Stylus)
    }
}
```

## 🔒 Security Considerations

### What This Protects Against

✅ **Classic Reentrancy**: Prevents recursive calls to protected functions  
✅ **Cross-Function Reentrancy**: Protects multiple functions with single guard  
✅ **Read-Only Reentrancy**: Prevents state inconsistencies during external calls  

### What This Doesn't Protect Against

❌ **Cross-Contract Reentrancy**: Multiple contracts with separate guards  
❌ **Delegatecall Attacks**: Different attack vector entirely  
❌ **Business Logic Errors**: Still need proper checks and state management  

### Best Practices

1. **Always use CEI Pattern**: Check → Effects → Interactions
2. **Update state before external calls**: Even with reentrancy protection
3. **Initialize the guard**: Call `init()` in your constructor
4. **Handle errors properly**: The guard can return errors
5. **Consider gas costs**: Protection adds minimal overhead

## 📖 Example: Vulnerable vs Safe

### ❌ Vulnerable (Without Protection)

```rust
pub fn withdraw_vulnerable(&mut self, amount: U256) -> Result<(), VaultError> {
    let balance = self.get_balance(msg::sender());
    
    if balance >= amount {
        // External call BEFORE state update - DANGEROUS!
        self.transfer_to(msg::sender(), amount)?;
        
        // Attacker can re-enter here and drain funds
        self.set_balance(msg::sender(), balance - amount);
    }
    
    Ok(())
}
```

### ✅ Safe (With ReentrancyGuard)

```rust
pub fn withdraw_safe(&mut self, amount: U256) -> Result<(), VaultError> {
    self.with_non_reentrant(|contract| {
        let balance = contract.get_balance(msg::sender());
        
        if balance >= amount {
            // Update state FIRST
            contract.set_balance(msg::sender(), balance - amount);
            
            // External call after state update - SAFE
            contract.transfer_to(msg::sender(), amount)?;
        }
        
        Ok(())
    })?
}
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

For Stylus-specific testing:

```bash
cargo stylus check
cargo test --features export-abi
```


## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for any new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license