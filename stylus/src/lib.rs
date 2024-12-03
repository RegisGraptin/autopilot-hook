// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(all(not(feature = "std"), not(feature = "export-abi")), no_main)]
extern crate alloc;


/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    prelude::entrypoint,
    stylus_proc::{public, sol_storage, SolidityError},
};

use alloy_primitives::{Address, U256, I256};
use alloy_sol_types::sol;

use stylus_sdk::storage::{StorageArray, StorageU256};

/// The currency data type.
pub type Currency = Address;

sol! {
    /// Indicates a custom error.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error CurveCustomError();
}

#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates a custom error.
    CustomError(CurveCustomError),
}

sol_storage! {
    #[entrypoint]
    struct UniswapCurve {
        // Store the previous volatility points
        StorageArray<StorageU256, 20> volatility_history;
        
        // Keep track of the first item of our sequence
        // Or the item to override to add a new one into our list
        StorageU256 index;
        
        // Count the number of items, usefull for initialization
        StorageU256 n_items;
    }
}

/// Interface of an [`UniswapCurve`] contract.
///
/// NOTE: The contract's interface can be modified in any way.
pub trait ICurve {

    /// Store and returns forcast volatility based on the new one provided.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `new_volatility` the new volatility computed.
    ///
    /// # Errors
    ///
    /// May return an [`Error`]. FIXME:
    ///
    /// # Events
    ///
    /// May emit any event. FIXME:

    fn forcast_volatility(
        &mut self,
        new_volatility: U256,
    ) -> Result<U256, Error>;

}

/// Declare that [`UniswapCurve`] is a contract
/// with the following external methods.
#[public]
impl ICurve for UniswapCurve {

    fn forcast_volatility(
        &mut self,
        new_volatility: U256,
    ) -> Result<U256, Error> {
        // At the first 20 volatility points, we return the imput data, as it is impossible for our model
        // to forcast the next point. 
        // When the first 20 data points are stored, we add at the index our new data, by override it as
        // we do not have pop(0) in solidity, so we are using the index mod 20 to keep track of our item.
        // Once store, we are using our trained model to forcast the data.

        // For the first 20 data, we cannot forcast volatility yet
        if self.n_items.lt(&U256::from(20)) {
            let mut index = self.volatility_history.get_mut(U256::from(*self.n_items)).unwrap();
            index.set(new_volatility);

            self.n_items.set(self.n_items.get() + U256::from(1));
            return Ok(new_volatility);
        }

        // We have at least 20 items, we can now forcast the volatility
        // We first store our new data
        let mut data = self.volatility_history.get_mut(self.index.get()).unwrap();
        data.set(new_volatility);

        let next_index = self.index.add_mod("1".parse::<U256>().unwrap(), "20".parse::<U256>().unwrap());
        self.index.set(next_index);

        // Forcast the volatility based on our trained ML model
        let volatility = self.compute_forcast_volatility()?;

        Ok(volatility)
    }

}

impl UniswapCurve {

    fn compute_forcast_volatility(
        &self,
    ) -> Result<U256, Error> {

        // Get the first item of our sequence
        let mut index = self.index.get();

        // To avoid floating numbers, we have scale the coefficient values
        let coefficient: Vec<I256> = vec![
            "-146693".parse::<I256>().unwrap(),
            "5476".parse::<I256>().unwrap(),
            "-132090".parse::<I256>().unwrap(),
            "80985".parse::<I256>().unwrap(),
            "-61377".parse::<I256>().unwrap(),
            "-18760".parse::<I256>().unwrap(),
            "113246".parse::<I256>().unwrap(),
            "-115346".parse::<I256>().unwrap(),
            "42895".parse::<I256>().unwrap(),
            "-54885".parse::<I256>().unwrap(),
            "94450".parse::<I256>().unwrap(),
            "-69297".parse::<I256>().unwrap(),
            "144672".parse::<I256>().unwrap(),
            "-83620".parse::<I256>().unwrap(),
            "-178140".parse::<I256>().unwrap(),
            "-19045".parse::<I256>().unwrap(),
            "55406".parse::<I256>().unwrap(),
            "62330".parse::<I256>().unwrap(),
            "-441246".parse::<I256>().unwrap(),
            "10240533".parse::<I256>().unwrap(),
        ];
        
        let mut prediction = "0".parse::<I256>().unwrap();

        for i in 0..20 {
            // Extract the value 
            let val = self.volatility_history.get(index).unwrap().to_string().parse::<I256>().unwrap();
            let coef = coefficient.get(i).unwrap();
            prediction = prediction + *coef * val;
            
            // Compute the new index
            index = index.add_mod(U256::from(1), U256::from(20));
        }

        // Rescale our prediction
        prediction = prediction / "10_000_000".parse::<I256>().unwrap();

        if prediction.le(&"0".parse::<I256>().unwrap()) {
            // Error in prediction, we cannot have a negative volatility
            return Err(Error::CustomError(CurveCustomError {}));
        }

        let forcast = prediction.abs().to_string().parse::<U256>().unwrap();

        Ok(forcast)
    }
}

/// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[motsu::test]
    fn forcast_volatility(contract: UniswapCurve) {

        let x_points: Vec<U256> = vec![
            "1660".parse::<U256>().unwrap(),
            "1673".parse::<U256>().unwrap(),
            "1681".parse::<U256>().unwrap(),
            "1721".parse::<U256>().unwrap(),
            "1626".parse::<U256>().unwrap(),
            "1574".parse::<U256>().unwrap(),
            "1587".parse::<U256>().unwrap(),
            "1550".parse::<U256>().unwrap(),
            "1566".parse::<U256>().unwrap(),
            "936".parse::<U256>().unwrap(),
            "857".parse::<U256>().unwrap(),
            "855".parse::<U256>().unwrap(),
            "1078".parse::<U256>().unwrap(),
            "1062".parse::<U256>().unwrap(),
            "1077".parse::<U256>().unwrap(),
            "1096".parse::<U256>().unwrap(),
            "1119".parse::<U256>().unwrap(),
            "1114".parse::<U256>().unwrap(),
            "1428".parse::<U256>().unwrap(),
            "1454".parse::<U256>().unwrap(),
        ];

        // Check the initialization
        for i in 0..20 {
            let volatility = contract
                .forcast_volatility(*x_points.get(i).unwrap())
                .expect("should compute `volatility`");
            assert_eq!(*x_points.get(i).unwrap(), volatility);
        }

        // Check model predicition
        let volatility = contract
                .forcast_volatility("1570".parse::<U256>().unwrap())
                .expect("should compute `volatility`");
        assert_eq!("1496".parse::<U256>().unwrap(), volatility);

    }
}
