#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;


extern "C" {
    fn getLightClientStoreSize() -> usize;
    fn initializeLightClientStoreCosmos(offset: *const u8, len: usize) -> *mut u8;
    fn processLightClientUpdate(offset: *const u8, len: usize, store: *const u8);
  }

#[ink::contract]
mod polka {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Polka {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl Polka {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let polka = Polka::default();
            assert_eq!(polka.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut polka = Polka::new(false);
            assert_eq!(polka.get(), false);
            polka.flip();
            assert_eq!(polka.get(), true);
        }
    }
}
