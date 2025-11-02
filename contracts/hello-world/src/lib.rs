#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, String, Vec,
};

#[contracttype]
#[derive(Clone)]
pub struct Employee {
    pub address: Address,
    pub salary: i128,
    pub name: String,
    pub active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct PayrollRecord {
    pub employee: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub period: String,
}

#[contracttype]
pub enum DataKey {
    Owner,
    Token,
    Employees,
    Employee(Address),
    PayrollHistory,
    NextEmployeeId,
}

#[contract]
pub struct PayrollContract;

#[contractimpl]
impl PayrollContract {
    /// Initialize the payroll contract with owner and payment token
    pub fn initialize(env: Env, owner: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic!("Contract already initialized");
        }
        
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::Token, &token);
        
        let employees: Vec<Address> = Vec::new(&env);
        env.storage().instance().set(&DataKey::Employees, &employees);
        
        let history: Vec<PayrollRecord> = Vec::new(&env);
        env.storage().instance().set(&DataKey::PayrollHistory, &history);
    }

    /// Add a new employee (only owner)
    pub fn add_employee(
        env: Env,
        caller: Address,
        employee_addr: Address,
        salary: i128,
        name: String,
    ) {
        caller.require_auth();
        Self::require_owner(&env, &caller);

        if salary <= 0 {
            panic!("Salary must be positive");
        }

        // Check if employee already exists
        if env.storage().instance().has(&DataKey::Employee(employee_addr.clone())) {
            panic!("Employee already exists");
        }

        let employee = Employee {
            address: employee_addr.clone(),
            salary,
            name,
            active: true,
        };

        // Store employee data
        env.storage()
            .instance()
            .set(&DataKey::Employee(employee_addr.clone()), &employee);

        // Add to employees list
        let mut employees: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Employees)
            .unwrap();
        employees.push_back(employee_addr);
        env.storage().instance().set(&DataKey::Employees, &employees);
    }

    /// Update employee salary (only owner)
    pub fn update_salary(env: Env, caller: Address, employee_addr: Address, new_salary: i128) {
        caller.require_auth();
        Self::require_owner(&env, &caller);

        if new_salary <= 0 {
            panic!("Salary must be positive");
        }

        let mut employee: Employee = env
            .storage()
            .instance()
            .get(&DataKey::Employee(employee_addr.clone()))
            .unwrap_or_else(|| panic!("Employee not found"));

        employee.salary = new_salary;
        env.storage()
            .instance()
            .set(&DataKey::Employee(employee_addr), &employee);
    }

    /// Deactivate an employee (only owner)
    pub fn deactivate_employee(env: Env, caller: Address, employee_addr: Address) {
        caller.require_auth();
        Self::require_owner(&env, &caller);

        let mut employee: Employee = env
            .storage()
            .instance()
            .get(&DataKey::Employee(employee_addr.clone()))
            .unwrap_or_else(|| panic!("Employee not found"));

        employee.active = false;
        env.storage()
            .instance()
            .set(&DataKey::Employee(employee_addr), &employee);
    }

    /// Process payroll for a single employee (only owner)
    pub fn pay_employee(env: Env, caller: Address, employee_addr: Address, period: String) {
        caller.require_auth();
        Self::require_owner(&env, &caller);

        let employee: Employee = env
            .storage()
            .instance()
            .get(&DataKey::Employee(employee_addr.clone()))
            .unwrap_or_else(|| panic!("Employee not found"));

        if !employee.active {
            panic!("Employee is not active");
        }

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .unwrap();

        let token_client = token::Client::new(&env, &token_address);

        // Transfer salary from contract to employee
        token_client.transfer(&caller, &employee_addr, &employee.salary);

        // Record payment in history
        let record = PayrollRecord {
            employee: employee_addr,
            amount: employee.salary,
            timestamp: env.ledger().timestamp(),
            period: period.clone(),
        };

        let mut history: Vec<PayrollRecord> = env
            .storage()
            .instance()
            .get(&DataKey::PayrollHistory)
            .unwrap();
        history.push_back(record);
        env.storage().instance().set(&DataKey::PayrollHistory, &history);
    }

    /// Process payroll for all active employees (only owner)
    pub fn pay_all_employees(env: Env, caller: Address, period: String) {
        caller.require_auth();
        Self::require_owner(&env, &caller);

        let employees: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Employees)
            .unwrap();

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .unwrap();

        let token_client = token::Client::new(&env, &token_address);

        for i in 0..employees.len() {
            let employee_addr = employees.get(i).unwrap();
            let employee: Employee = env
                .storage()
                .instance()
                .get(&DataKey::Employee(employee_addr.clone()))
                .unwrap();

            if employee.active {
                // Transfer salary
                token_client.transfer(&caller, &employee_addr, &employee.salary);

                // Record payment
                let record = PayrollRecord {
                    employee: employee_addr.clone(),
                    amount: employee.salary,
                    timestamp: env.ledger().timestamp(),
                    period: period.clone(),
                };

                let mut history: Vec<PayrollRecord> = env
                    .storage()
                    .instance()
                    .get(&DataKey::PayrollHistory)
                    .unwrap();
                history.push_back(record);
                env.storage().instance().set(&DataKey::PayrollHistory, &history);
            }
        }
    }

    /// Get employee details
    pub fn get_employee(env: Env, employee_addr: Address) -> Employee {
        env.storage()
            .instance()
            .get(&DataKey::Employee(employee_addr))
            .unwrap_or_else(|| panic!("Employee not found"))
    }

    /// Get all employees
    pub fn get_all_employees(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Employees)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get payroll history
    pub fn get_payroll_history(env: Env) -> Vec<PayrollRecord> {
        env.storage()
            .instance()
            .get(&DataKey::PayrollHistory)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Get contract owner
    pub fn get_owner(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .unwrap()
    }

    // Helper function to check if caller is owner
    fn require_owner(env: &Env, caller: &Address) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        if caller != &owner {
            panic!("Only owner can perform this action");
        }
    }
}

#[cfg(test)]
mod test;