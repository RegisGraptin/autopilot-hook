// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(all(not(feature = "std"), not(feature = "export-abi")), no_main)]
extern crate alloc;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    evm,
    prelude::entrypoint,
    stylus_proc::{public, sol_storage, SolidityError},
};

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

/// The currency data type.
pub type Currency = Address;

sol! {
    /// Emitted when the amount of input tokens for an exact-output swap
    /// is calculated.
    #[allow(missing_docs)]
    event AmountInCalculated(
        uint256 amount_out,
        address input,
        address output,
        bool zero_for_one
    );

    /// Emitted when the amount of output tokens for an exact-input swap
    /// is calculated.
    #[allow(missing_docs)]
    event AmountOutCalculated(
        uint256 amount_in,
        address input,
        address output,
        bool zero_for_one
    );
}

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
    struct UniswapCurve { }
}

/// Interface of an [`UniswapCurve`] contract.
///
/// NOTE: The contract's interface can be modified in any way.
pub trait ICurve {
    /// Returns the amount of input tokens for an exact-output swap.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `amount_out` the amount of output tokens the user expects to receive.
    /// * `input` - The input token.
    /// * `output` - The output token.
    /// * `zero_for_one` - True if the input token is token0.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    ///
    /// # Events
    ///
    /// May emit any event.
    fn get_amount_in_for_exact_output(
        &mut self,
        amount_out: U256,
        input: Currency,
        output: Currency,
        zero_for_one: bool,
    ) -> Result<U256, Error>;

    /// Returns the amount of output tokens for an exact-input swap.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `amount_in` - The amount of input tokens.
    /// * `input` - The input token.
    /// * `output` - The output token.
    /// * `zero_for_one` - True if the input token is `token_0`.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    ///
    /// # Events
    ///
    /// May emit any event.
    fn get_amount_out_from_exact_input(
        &mut self,
        amount_in: U256,
        input: Currency,
        output: Currency,
        zero_for_one: bool,
    ) -> Result<U256, Error>;
}

/// Declare that [`UniswapCurve`] is a contract
/// with the following external methods.
#[public]
impl ICurve for UniswapCurve {
    fn get_amount_in_for_exact_output(
        &mut self,
        amount_out: U256,
        input: Currency,
        output: Currency,
        zero_for_one: bool,
    ) -> Result<U256, Error> {
        // Calculate `amount_in` based on swap params.
        let amount_in = self.calculate_amount_in(amount_out, input, output, zero_for_one)?;

        // Emit an event if needed.
        evm::log(AmountInCalculated {
            amount_out,
            input,
            output,
            zero_for_one,
        });

        Ok(amount_in)
    }

    fn get_amount_out_from_exact_input(
        &mut self,
        amount_in: U256,
        input: Currency,
        output: Currency,
        zero_for_one: bool,
    ) -> Result<U256, Error> {
        // Calculate `amount_out` based on swap params.
        let amount_out = self.calculate_amount_out(amount_in, input, output, zero_for_one)?;

        // Emit an event if needed.
        evm::log(AmountOutCalculated {
            amount_in,
            input,
            output,
            zero_for_one,
        });

        Ok(amount_out)
    }
}

impl UniswapCurve {
    /// Calculates the amount of input tokens for an exact-output swap.
    ///
    /// # Arguments
    ///
    /// * `&self` - Read access to the contract's state.
    /// * `amount_out` the amount of output tokens the user expects to receive.
    /// * `input` - The input token.
    /// * `output` - The output token.
    /// * `zero_for_one` - True if the input token is token0.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn calculate_amount_in(
        &self,
        amount_out: U256,
        _input: Currency,
        _output: Currency,
        _zero_for_one: bool,
    ) -> Result<U256, Error> {
        // This is an example of a constant-sum swap curve,
        // tokens are traded exactly 1:1.
        //
        // You can implement any swap curve.
        let amount_in = amount_out;

        Ok(amount_in)
    }

    /// Returns the amount of output tokens for an exact-input swap.
    ///
    /// # Arguments
    ///
    /// * `&mut self` - Write access to the contract's state.
    /// * `amount_in` - The amount of input tokens.
    /// * `input` - The input token.
    /// * `output` - The output token.
    /// * `zero_for_one` - True if the input token is `token_0`.
    ///
    /// # Errors
    ///
    /// May return an [`Error`].
    fn calculate_amount_out(
        &self,
        amount_in: U256,
        _input: Currency,
        _output: Currency,
        _zero_for_one: bool,
    ) -> Result<U256, Error> {
        // This is an example of a constant-sum swap curve,
        // tokens are traded exactly 1:1.
        //
        // You can implement any swap curve.
        let amount_out = amount_in;

        Ok(amount_out)
    }
}

/// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, uint};

    const CURRENCY_1: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const CURRENCY_2: Address = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    #[test]
    fn sample_test() {
        assert_eq!(4, 2 + 2);
    }

    #[motsu::test]
    fn calculates_amount_in(contract: UniswapCurve) {
        let amount_out = uint!(1_U256);
        let expected_amount_in = amount_out; // 1:1 swap
        let amount_in = contract
            .calculate_amount_in(amount_out, CURRENCY_1, CURRENCY_2, true)
            .expect("should calculate `amount_in`");
        assert_eq!(expected_amount_in, amount_in);
    }

    #[motsu::test]
    fn calculates_amount_out(contract: UniswapCurve) {
        let amount_in = uint!(2_U256);
        let expected_amount_out = amount_in; // 1:1 swap
        let amount_out = contract
            .calculate_amount_out(amount_in, CURRENCY_1, CURRENCY_2, true)
            .expect("should calculate `amount_out`");
        assert_eq!(expected_amount_out, amount_out);
    }

    #[motsu::test]
    fn returns_amount_in_for_exact_output(contract: UniswapCurve) {
        let amount_out = uint!(1_U256);
        let expected_amount_in = amount_out; // 1:1 swap
        let amount_in = contract
            .get_amount_in_for_exact_output(amount_out, CURRENCY_1, CURRENCY_2, true)
            .expect("should calculate `amount_in`");
        assert_eq!(expected_amount_in, amount_in);
    }

    #[motsu::test]
    fn returns_amount_out_from_exact_input(contract: UniswapCurve) {
        let amount_in = uint!(2_U256);
        let expected_amount_out = amount_in; // 1:1 swap
        let amount_out = contract
            .get_amount_out_from_exact_input(amount_in, CURRENCY_1, CURRENCY_2, true)
            .expect("should calculate `amount_out`");
        assert_eq!(expected_amount_out, amount_out);
    }
}
