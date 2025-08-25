//! Binary entry point for the reentrancy-guard-stylus project
//! 
//! This file serves as the main entry point when the contract is compiled
//! as a binary for deployment with Stylus.

#![cfg_attr(not(feature = "export-abi"), no_main)]

use reentrancy_guard_stylus::VaultContract;

// Export the contract for ABI generation
stylus_sdk::entrypoint!(VaultContract);