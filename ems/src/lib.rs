extern crate alloc;

use stylus_sdk::{
    prelude::*,
};
use alloy_primitives::{U256, Address};
use alloy_sol_types::sol;
use alloc::{vec::Vec, string::String};

sol_storage! {
    #[entrypoint]
    pub struct EmployeeManagement {
        address admin;
        uint256 employee_count;
        mapping(address => Employee) employees;
        address[] employee_addresses;
        mapping(uint256 => address[]) department_employees;
        uint256 salary_budget;
        uint256 total_salaries;
    }

    pub struct Employee {
        uint256 id;
        address employee_address;
        bytes name;
        uint256 department;
        uint256 salary;
        uint256 hire_date;
        bool is_active;
        uint256 total_earned;
    }
}

#[public]
impl EmployeeManagement {
    /// Initialize employee management system
    pub fn new(&mut self, initial_budget: U256) -> Result<(), Vec<u8>> {
        self.admin.set(self.vm().msg_sender());
        self.employee_count.set(U256::from(0));
        self.salary_budget.set(initial_budget);
        self.total_salaries.set(U256::from(0));
        
        log(self.vm(), SystemInitialized {
            admin: self.vm().msg_sender(),
            initial_budget,
        });
        
        Ok(())
    }

    /// Add new employee (admin only)
    pub fn add_employee(
        &mut self,
        employee_address: Address,
        name: String,
        department: U256,
        salary: U256,
    ) -> Result<U256, Vec<u8>> {
        self.only_admin()?;
        
        if employee_address == Address::ZERO {
            return Err("Invalid employee address".as_bytes().to_vec());
        }
        
        if self.employees.get(employee_address).is_active.get() {
            return Err("Employee already exists".as_bytes().to_vec());
        }
        
        if salary == U256::from(0) {
            return Err("Salary must be greater than zero".as_bytes().to_vec());
        }
        
        // Check budget
        let new_total = self.total_salaries.get() + salary;
        if new_total > self.salary_budget.get() {
            return Err("Exceeds salary budget".as_bytes().to_vec());
        }
        
        let employee_id = self.employee_count.get() + U256::from(1);
        let hire_date = U256::from(self.vm().block_timestamp());
        
        let mut employee = self.employees.setter(employee_address);
        employee.id.set(employee_id);
        employee.employee_address.set(employee_address);
        employee.name.set_bytes(name.as_bytes());
        employee.department.set(department);
        employee.salary.set(salary);
        employee.hire_date.set(hire_date);
        employee.is_active.set(true);
        employee.total_earned.set(U256::from(0));
        
        self.employee_addresses.push(employee_address);
        self.department_employees.setter(department).push(employee_address);
        self.employee_count.set(employee_id);
        self.total_salaries.set(new_total);
        
        log(self.vm(), EmployeeAdded {
            employee_id,
            employee_address,
            department,
            salary,
        });
        
        Ok(employee_id)
    }

    /// Update employee salary (admin only)
    pub fn update_salary(&mut self, employee_address: Address, new_salary: U256) -> Result<(), Vec<u8>> {
        self.only_admin()?;
        
        let employee = self.employees.get(employee_address);
        if !employee.is_active.get() {
            return Err("Employee not found or inactive".as_bytes().to_vec());
        }
        
        if new_salary == U256::from(0) {
            return Err("Salary must be greater than zero".as_bytes().to_vec());
        }
        
        // Check budget with salary change
        let current_total = self.total_salaries.get();
        let old_salary = employee.salary.get();
        let salary_difference = if new_salary > old_salary {
            new_salary - old_salary
        } else {
            U256::from(0)
        };
        
        if current_total + salary_difference > self.salary_budget.get() {
            return Err("Salary update exceeds budget".as_bytes().to_vec());
        }
        
        // Update salary
        self.employees.setter(employee_address).salary.set(new_salary);
        
        // Update total salaries
        let new_total = current_total - old_salary + new_salary;
        self.total_salaries.set(new_total);
        
        log(self.vm(), SalaryUpdated {
            employee_address,
            old_salary,
            new_salary,
        });
        
        Ok(())
    }


    /// Pay salary to employee
    pub fn pay_salary(&mut self, employee_address: Address) -> Result<(), Vec<u8>> {
        self.only_admin()?;
        
        let employee = self.employees.get(employee_address);
        if !employee.is_active.get() {
            return Err("Employee not found or inactive".as_bytes().to_vec());
        }
        
        // In real implementation, would transfer tokens/ETH
        let salary = employee.salary.get();
        let current_earned = employee.total_earned.get();
        let new_total_earned = current_earned + salary;
        
        self.employees.setter(employee_address).total_earned.set(new_total_earned);
        
        log(self.vm(), SalaryPaid {
            employee_address,
            amount: salary,
            total_earned: new_total_earned,
        });
        
        Ok(())
    }

    /// Terminate employee (admin only)
    pub fn terminate_employee(&mut self, employee_address: Address) -> Result<(), Vec<u8>> {
        self.only_admin()?;
        
        // First check if employee exists and get salary
        let (is_active, salary) = {
            let employee = self.employees.get(employee_address);
            (employee.is_active.get(), employee.salary.get())
        };
        
        if !is_active {
            return Err("Employee not found or already terminated".as_bytes().to_vec());
        }
        
        // Update employee status
        self.employees.setter(employee_address).is_active.set(false);
        
        // Update total salaries budget
        let new_total = self.total_salaries.get() - salary;
        self.total_salaries.set(new_total);
        
        log(self.vm(), EmployeeTerminated {
            employee_address,
            termination_date: U256::from(self.vm().block_timestamp()),
        });
        
        Ok(())
    }

    /// Get employee details
    pub fn get_employee(&self, employee_address: Address) -> (U256, Address, Vec<u8>, U256, U256, U256, bool, U256) {
        let employee = self.employees.get(employee_address);
        (
            employee.id.get(),
            employee.employee_address.get(),
            employee.name.get_bytes(),
            employee.department.get(),
            employee.salary.get(),
            employee.hire_date.get(),
            employee.is_active.get(),
            employee.total_earned.get(),
        )
    }

    /// Check if employee exists and is active
    pub fn is_active_employee(&self, employee_address: Address) -> bool {
        self.employees.get(employee_address).is_active.get()
    }

    /// Get total active employees (simplified)
    pub fn get_active_employee_count(&self) -> U256 {
        // In real implementation, would maintain active count
        self.employee_count.get()
    }

    /// Update salary budget (admin only)
    pub fn update_budget(&mut self, new_budget: U256) -> Result<(), Vec<u8>> {
        self.only_admin()?;
        
        if new_budget < self.total_salaries.get() {
            return Err("New budget cannot be less than current total salaries".as_bytes().to_vec());
        }
        
        let old_budget = self.salary_budget.get();
        self.salary_budget.set(new_budget);
        
        log(self.vm(), BudgetUpdated {
            old_budget,
            new_budget,
        });
        
        Ok(())
    }

   
    /// Get admin address
    pub fn get_admin(&self) -> Address {
        self.admin.get()
    }


    // Internal functions
    fn only_admin(&self) -> Result<(), Vec<u8>> {
        if self.vm().msg_sender() != self.admin.get() {
            return Err("Only admin can perform this action".as_bytes().to_vec());
        }
        Ok(())
    }

}

sol! {
    event SystemInitialized(address indexed admin, uint256 initial_budget);
    event EmployeeAdded(uint256 indexed employee_id, address indexed employee_address, uint256 indexed department, uint256 salary);
    event SalaryUpdated(address indexed employee_address, uint256 old_salary, uint256 new_salary);
    event SalaryPaid(address indexed employee_address, uint256 amount, uint256 total_earned);
    event EmployeeTerminated(address indexed employee_address, uint256 termination_date);
    event BudgetUpdated(uint256 old_budget, uint256 new_budget);
}