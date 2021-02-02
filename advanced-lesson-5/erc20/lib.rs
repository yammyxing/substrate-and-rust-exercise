#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    use ink_storage::collections::HashMap as StorageHashMap;

    #[ink(storage)]
    pub struct Erc20 {
        total_supply: Balance,
        balances: StorageHashMap<AccountId, Balance>,
        allowance: StorageHashMap<(AccountId, AccountId), Balance>
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InSufficientBalance,
        NotEnoughAllowance
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, total_supply);
            let instance = Self {
                total_supply,
                balances,
                allowance: StorageHashMap::new()
            };

            instance
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            *self.balances.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn allowance_of(&self, owner: AccountId, spender: AccountId) -> Balance {
            *self.allowance.get(&(owner, spender)).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let who = Self::env().caller();
            self._transfer_from_to(who, to, value)
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let who = Self::env().caller();
            let from_allowance = self.allowance_of(from, who);
            if from_allowance < value {
                return Err(Error::NotEnoughAllowance);
            }
            self._transfer_from_to(from, to, value)?;
            self.allowance.insert((from, who), from_allowance - value);
            self.env().emit_event(Approval {
                owner: from,
                spender: to,
                value
            });
            Ok(())
        }

        // #[ink(message)]
        // pub fn burn() {

        // }

        // #[ink(message)]
        // pub fn issue() {

        // }

        fn _transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InSufficientBalance);
            }
            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);

            self.env().emit_event(Transfer {
                from,
                to,
                value
            });

            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the create contract works.
        #[test]
        fn create_contract_works() {
            let erc20 = Erc20::new(1000);
            assert_eq!(erc20.total_supply(), 1000);
        }

        #[test]
        fn balance_of_works() {
            let erc20 = Erc20::new(1000);
            let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Can't get accounts");
            assert_eq!(erc20.balance_of(accounts.alice), 1000);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
        }
    }
}
