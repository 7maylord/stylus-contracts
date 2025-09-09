extern crate alloc;

use stylus_sdk::{
    prelude::*,
    msg, block
};

use alloc::{vec::Vec, string::String};
use alloy_primitives::{U256, Address};

sol_storage! {
    #[entrypoint]
    pub struct EmployeeManagement {
        address admin;
        uint256 employee_count;
        mapping(address => Employee) employees;
        address[] employee_addresses;
        mapping(uint256 => address[]) department_employees;
    }

    pub struct Employee {
        uint256 id;
        address employee_address;
        bytes name;
        uint256 department;
        uint256 hire_date;
        bool is_active;
    }
}

#[public]
impl EmployeeManagement {
    /// Initialize employee management system
    pub fn new(&mut self) -> Result<(), Vec<u8>> {
        self.admin.set(msg::sender());
        self.employee_count.set(U256::from(0));
        
        Ok(())
    }

    /// Add new employee (admin only)
    pub fn add_employee(
        &mut self,
        employee_address: Address,
        name: String,
        department: U256,
    ) -> Result<U256, Vec<u8>> {
        self.only_admin()?;
        
        if employee_address == Address::ZERO {
            return Err("Invalid employee address".as_bytes().to_vec());
        }
        
        if self.employees.get(employee_address).is_active.get() {
            return Err("Employee already exists".as_bytes().to_vec());
        }
        
        
        let employee_id = self.employee_count.get() + U256::from(1);
        let hire_date = U256::from(block::timestamp());
        
        let mut employee = self.employees.setter(employee_address);
        employee.id.set(employee_id);
        employee.employee_address.set(employee_address);
        employee.name.set_bytes(name.as_bytes());
        employee.department.set(department);
        employee.hire_date.set(hire_date);
        employee.is_active.set(true);
        
        self.employee_addresses.push(employee_address);
        self.department_employees.setter(department).push(employee_address);
        self.employee_count.set(employee_id);
        
        Ok(employee_id)
    }

    /// Terminate employee (admin only)
    pub fn terminate_employee(&mut self, employee_address: Address) -> Result<(), Vec<u8>> {
        self.only_admin()?;
        
        // check if employee exists
        let is_active = {
            let employee = self.employees.get(employee_address);
            employee.is_active.get()
        };
        
        if !is_active {
            return Err("Employee not found or already terminated".as_bytes().to_vec());
        }
        
        // Update employee status
        self.employees.setter(employee_address).is_active.set(false);
        
        Ok(())
    }

    /// Get employee details
    pub fn get_employee(&self, employee_address: Address) -> (U256, Address, Vec<u8>, U256, U256, bool) {
        let employee = self.employees.get(employee_address);
        (
            employee.id.get(),
            employee.employee_address.get(),
            employee.name.get_bytes(),
            employee.department.get(),
            employee.hire_date.get(),
            employee.is_active.get(),
        )
    }

    /// Get admin address
    pub fn get_admin(&self) -> Address {
        self.admin.get()
    }

    /// Check if employee exists and is active
    pub fn is_active_employee(&self, employee_address: Address) -> bool {
        self.employees.get(employee_address).is_active.get()
    }

    // Internal functions
    fn only_admin(&self) -> Result<(), Vec<u8>> {
        if msg::sender() != self.admin.get() {
            return Err("Only admin can perform this action".as_bytes().to_vec());
        }
        Ok(())
    }

}

