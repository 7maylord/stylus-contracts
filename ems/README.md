# Employee Management System (EMS)

A comprehensive employee management smart contract written in Rust for Arbitrum Stylus. This contract provides a decentralized solution for managing employees, salaries, and departmental organization with built-in budget controls and administrative features.

## Features

- **Employee Management**: Add, update, and terminate employees
- **Salary Administration**: Set and update employee salaries with budget validation
- **Budget Control**: Maintain and enforce salary budget constraints
- **Department Organization**: Organize employees by departments
- **Payment Tracking**: Track total earnings per employee
- **Admin Controls**: Secure administrative functions with access control
- **Event Logging**: Comprehensive event emission for all major actions

## Contract Structure

The contract manages the following key data structures:

- **Employee Records**: ID, address, name, department, salary, hire date, status, and total earnings
- **Budget Management**: Total budget allocation and current salary commitments
- **Department Mapping**: Track employees by department for organizational purposes

## Main Functions

### Administrative Functions (Admin Only)
- `new(initial_budget)` - Initialize the contract with a salary budget
- `add_employee(address, name, department, salary)` - Add a new employee
- `update_salary(address, new_salary)` - Update an employee's salary
- `pay_salary(address)` - Process salary payment (updates total earned)
- `terminate_employee(address)` - Deactivate an employee
- `update_budget(new_budget)` - Modify the total salary budget

### Public View Functions
- `get_employee(address)` - Retrieve complete employee information
- `is_active_employee(address)` - Check if an employee is active
- `get_active_employee_count()` - Get total number of employees
- `get_admin()` - Get the contract administrator address

## Quick Start 

Install [Rust](https://www.rust-lang.org/tools/install), and then install the Stylus CLI tool with Cargo

```bash
cargo install --force cargo-stylus cargo-stylus-check
```

Add the `wasm32-unknown-unknown` build target to your Rust compiler:

```
rustup target add wasm32-unknown-unknown
```

You should now have it available as a Cargo subcommand:

```bash
cargo stylus --help
```

Then, clone this repository:

```bash
git clone <repository-url> && cd stylus-contracts/ems
```

### Testnet Information

All testnet information, including faucets and RPC endpoints can be found [here](https://docs.arbitrum.io/stylus/reference/testnet-information).

### ABI Export

You can export the Solidity ABI for your program by using the `cargo stylus` tool as follows:

```bash
cargo stylus export-abi
```

Exporting ABIs uses a feature that is enabled by default in your Cargo.toml:

```toml
[features]
export-abi = ["stylus-sdk/export-abi"]
```

## Deploying

You can use the `cargo stylus` command to also deploy your program to the Stylus testnet. We can use the tool to first check
our program compiles to valid WASM for Stylus and will succeed a deployment onchain without transacting. By default, this will use the Stylus testnet public RPC endpoint. See here for [Stylus testnet information](https://docs.arbitrum.io/stylus/reference/testnet-information)

```bash
cargo stylus check
```

If successful, Here's how to deploy:

```bash
cargo stylus deploy \
    --endpoint <youRPCurl> \
    --private-key <yourPrivateKey> \
    --constructor-args '["<initialbudgetamount>"]' 
```

## Events

The contract emits the following events for transparency and off-chain monitoring:

- `SystemInitialized(admin, initial_budget)` - Contract initialization
- `EmployeeAdded(employee_id, employee_address, department, salary)` - New employee added
- `SalaryUpdated(employee_address, old_salary, new_salary)` - Salary modification
- `SalaryPaid(employee_address, amount, total_earned)` - Salary payment processed
- `EmployeeTerminated(employee_address, termination_date)` - Employee deactivated
- `BudgetUpdated(old_budget, new_budget)` - Budget modification


## License

This project is fully open source, including an Apache-2.0 or MIT license at your choosing under your own copyright.
