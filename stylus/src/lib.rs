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

// sol! {
//     // Example event
//     #[allow(missing_docs)]
//     event AmountInCalculated(
//         uint256 amount_out,
//         address input,
//         address output,
//         bool zero_for_one
//     );
// }

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

// sol_storage! {
//     #[entrypoint]
//     struct UniswapCurve {
//         volatility_history: StorageVec<I256>
//     }
// }

sol_storage! {
    #[entrypoint]
    struct UniswapCurve {
        StorageArray<StorageU256, 20> volatility_history;
        StorageU256 index;
        StorageU256 n_data;
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


        // For the first 20 data, we cannot forcast volatility yet
        // So we are returning the current volatility
        let current_n_data = self.n_data.get();

        if current_n_data < "20".parse::<U256>().unwrap() {
            let mut index = self.volatility_history.get_mut(current_n_data).unwrap();
            index.set(new_volatility);

            let new_n_data = current_n_data + "1".parse::<U256>().unwrap();
            self.n_data.set(new_n_data);

            return Ok(new_volatility);
        }

        // // We have 20 items in our array
        // // We need to remove the first one and add the new one to the end
        // let d = self.volatility_history.load_mut();

        // New item will be store on the first one

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

        let mut index = self.index.get();

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

        for i in 0..19 {

            // Compute the new index
            index = index.add_mod(U256::from(i) , U256::from(20));
            
            // Extract the value 
            let val = self.volatility_history.get(index).unwrap().to_string().parse::<I256>().unwrap();

            let coef = coefficient.get(i).unwrap();
            
            prediction = prediction + *coef * val;
        }

        if prediction.le(&"0".parse::<I256>().unwrap()) {
            // Error in prediction, we cannot have a negative volatility
        }

        let forcast = prediction.abs().to_string().parse::<U256>().unwrap();

        Ok(forcast)
    }
}

// /// Unit tests
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use alloy_primitives::{address, uint};

//     const CURRENCY_1: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
//     const CURRENCY_2: Address = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

//     #[test]
//     fn sample_test() {
//         assert_eq!(4, 2 + 2);
//     }

//     #[motsu::test]
//     fn calculates_amount_in(contract: UniswapCurve) {
//         let amount_out = uint!(1_U256);
//         let expected_amount_in = amount_out; // 1:1 swap
//         let amount_in = contract
//             .calculate_amount_in(amount_out, CURRENCY_1, CURRENCY_2, true)
//             .expect("should calculate `amount_in`");
//         assert_eq!(expected_amount_in, amount_in);
//     }

//     #[motsu::test]
//     fn calculates_amount_out(contract: UniswapCurve) {
//         let amount_in = uint!(2_U256);
//         let expected_amount_out = amount_in; // 1:1 swap
//         let amount_out = contract
//             .calculate_amount_out(amount_in, CURRENCY_1, CURRENCY_2, true)
//             .expect("should calculate `amount_out`");
//         assert_eq!(expected_amount_out, amount_out);
//     }

//     #[motsu::test]
//     fn returns_amount_in_for_exact_output(contract: UniswapCurve) {
//         let amount_out = uint!(1_U256);
//         let expected_amount_in = amount_out; // 1:1 swap
//         let amount_in = contract
//             .get_amount_in_for_exact_output(amount_out, CURRENCY_1, CURRENCY_2, true)
//             .expect("should calculate `amount_in`");
//         assert_eq!(expected_amount_in, amount_in);
//     }

//     #[motsu::test]
//     fn returns_amount_out_from_exact_input(contract: UniswapCurve) {
//         let amount_in = uint!(2_U256);
//         let expected_amount_out = amount_in; // 1:1 swap
//         let amount_out = contract
//             .get_amount_out_from_exact_input(amount_in, CURRENCY_1, CURRENCY_2, true)
//             .expect("should calculate `amount_out`");
//         assert_eq!(expected_amount_out, amount_out);
//     }
// }
