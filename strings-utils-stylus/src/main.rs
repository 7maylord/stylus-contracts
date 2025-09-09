#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use alloy_primitives::{Address, I256, U256};
use alloy_sol_types::sol;
use stylus_sdk::{entrypoint, prelude::*};

// Import the strings utility functions
use strings_utils_stylus::{
    address_to_checksum_hex_string, address_to_hex_string, to_hex_string, to_hex_string_with_length,
    to_string, to_string_signed, StringsError,
};

// Define the contract's external interface
sol! {
    interface IStringsUtils {
        function toString(uint256 value) external pure returns (string memory);
        function toStringSigned(int256 value) external pure returns (string memory);
        function toHexString(uint256 value) external pure returns (string memory);
        function toHexStringWithLength(uint256 value, uint256 length) external pure returns (string memory);
        function addressToHexString(address addr) external pure returns (string memory);
        function addressToChecksumHexString(address addr) external pure returns (string memory);
    }
}

// The main contract struct
#[entrypoint]
#[storage]
pub struct StringsUtils;

// Define custom errors for the contract
sol! {
    error InsufficientHexLength(uint256 value, uint256 length);
}

#[public]
impl StringsUtils {
    /// Converts a U256 value to its ASCII decimal string representation
    pub fn to_string(&self, value: U256) -> String {
        to_string(value)
    }

    /// Converts an I256 value to its ASCII decimal string representation
    pub fn to_string_signed(&self, value: I256) -> String {
        to_string_signed(value)
    }

    /// Converts a U256 value to its ASCII hexadecimal string representation
    pub fn to_hex_string(&self, value: U256) -> String {
        to_hex_string(value)
    }

    /// Converts a U256 value to its ASCII hexadecimal string representation with fixed length
    pub fn to_hex_string_with_length(&self, value: U256, length: U256) -> Result<String, Vec<u8>> {
        let length_usize = length.to::<usize>();
        match to_hex_string_with_length(value, length_usize) {
            Ok(result) => Ok(result),
            Err(StringsError::InsufficientHexLength { value, length }) => {
                Err(InsufficientHexLength { value, length: U256::from(length) }.encode())
            }
        }
    }

    /// Converts an Address to its ASCII hexadecimal string representation
    pub fn address_to_hex_string(&self, addr: Address) -> String {
        address_to_hex_string(addr)
    }

    /// Converts an Address to its checksummed ASCII hexadecimal string representation
    pub fn address_to_checksum_hex_string(&self, addr: Address) -> String {
        address_to_checksum_hex_string(addr)
    }
}

#[cfg(feature = "export-abi")]
fn main() {
    strings_utils_stylus::print_from_args();
}
