//! Astro-float (astronomically large floating point numbers) is a library that implements arbitrary precision floating point numbers.
//!
//! ## Introduction
//!
//! **Numbers**
//!
//!
//! The number is defined by the data type `BigFloat`.
//! Each finite number consists of an array of words representing the mantissa, exponent, and sign.
//! `BigFloat` can also be `Inf` (positive infinity), `-Inf` (negative infinity) or `NaN` (not-a-number).
//!
//!
//! `BigFloat` creation operations take bit precision as an argument.
//! Precision is always rounded up to the nearest word.
//! For example, if you specify a precision of 1 bit, then it will be converted to 64 bits when one word has a size of 64 bits.
//! If you specify a precision of 65 bits, the resulting precision will be 128 bits (2 words), and so on.
//!
//!
//! Most operations take the rounding mode as an argument.
//! The operation will typically internally result in a number with more precision than necessary.
//! Before the result is returned to the user, the result is rounded according to the rounding mode and reduced to the expected precision.
//!
//!
//! `BigFloat` can be parsed from a string and formatted into a string using binary, octal, decimal, or hexadecimal representation.
//!
//!
//! Numbers can be subnormal. Usually any number is normalized: the most significant bit of the mantissa is set to 1.
//! If the result of the operation has the smallest possible exponent, then normalization cannot be performed,
//! and some significant bits of the mantissa may become 0. This allows for a more gradual transition to zero.
//!
//! **Error handling**
//!
//! In case of an error, such as memory allocation error, `BigFloat` takes the value `NaN`.
//! `BigFloat::get_err()` can be used to get the associated error in this situation.
//!
//! **BigFloatNumber**
//!
//! `BigFloatNumber` is a data type provided for backward compatibility.
//! It represents the finite number, i.e. it has mantissa, sign, and exponent, and can't be `Inf` or `NaN`.
//! `BigFloat` uses `BigFloatNumber` internally. It is advisable to use `BigFloat` instead of `BigFloatNumber`.
//!
//! **Constants**
//!
//! Constants such as pi or the Euler number have arbitrary precision and are evaluated lazily and then cached in the constants cache.
//! Some functions expect constants cache as parameter because the library does not maintain global state.
//!
//!  
//! ## Examples
//!
//! ``` rust
//! use astro_float::BigFloat;
//! use astro_float::Consts;
//! use astro_float::RoundingMode;
//!
//! // Precision with some space for error.
//! let p = 1024 + 8;
//!
//! // Rounding of all operations
//! let rm = RoundingMode::ToEven;
//!
//! // Initialize mathematical constants cache
//! let mut cc = Consts::new().unwrap();
//!
//! // Compute pi: pi = 6*arctan(1/sqrt(3))
//! let six = BigFloat::from_word(6, 1);
//! let three = BigFloat::from_word(3, p);
//!
//! let n = three.sqrt(p, rm);
//! let n = n.reciprocal(p, rm);
//! let n = n.atan(p, rm, &mut cc);
//! let mut pi = six.mul(&n, p, rm);
//!
//! // Reduce precision to 1024
//! pi.set_precision(1024, rm).expect("Precision updated");
//!
//! // Use library's constant for verifying the result
//! let pi_lib = cc.pi(1024, rm).unwrap().into();
//!
//! // Compare computed constant with library's constant
//! assert_eq!(pi.cmp(&pi_lib), Some(0));
//!
//! // Print using decimal radix.
//! println!("{}", pi);
//!
//! // output: 3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706798214808651328230664709384460955058223172535940812848111745028410270193852110555964462294895493038196442881097566593344612847564823378678316527120190914564856692346034861045432664821339360726024914127372458699748e+0
//! ```
//!
//! ## no_std
//!
//! The library can work without the standard library provided there is a memory allocator. The standard library dependency is activated by the feature `std`.
//! The feature `std` is active by default and must be excluded when specifying dependency, e.g.:
//!
//! ``` toml
//! [dependencies]
//! astro-float = { version = "0.3.2", default-features = false }
//! ```
//!

#![deny(missing_docs)]
#![deny(clippy::suspicious)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod common;
mod conv;
//mod ctx;
mod defs;
mod ext;
mod for_3rd;
mod mantissa;
mod num;
mod ops;
mod parser;
mod strop;

//pub use crate::ctx::with_consts;
//pub use crate::ctx::with_precision;
//pub use crate::ctx::with_rounding_mode;
//pub use crate::ctx::with_value;
//pub use crate::ctx::Context;
pub use crate::defs::Error;
pub use crate::defs::Exponent;
pub use crate::defs::Radix;
pub use crate::defs::RoundingMode;
pub use crate::defs::Sign;
pub use crate::defs::Word;
pub use crate::ext::BigFloat;
pub use crate::ext::INF_NEG;
pub use crate::ext::INF_POS;
pub use crate::ext::NAN;
pub use crate::num::BigFloatNumber;
pub use crate::ops::consts::Consts;

pub use crate::defs::EXPONENT_MAX;
pub use crate::defs::EXPONENT_MIN;
pub use crate::defs::WORD_BIT_SIZE;

#[cfg(test)]
mod tests {

    #[test]
    fn test_bigfloat() {
        use crate::BigFloat;
        use crate::Consts;
        use crate::RoundingMode;

        // Precision with some space for error.
        let p = 1024 + 8;

        // Rounding of all operations
        let rm = RoundingMode::ToEven;

        // Initialize mathematical constants cache
        let mut cc = Consts::new().unwrap();

        // Compute pi: pi = 6*arctan(1/sqrt(3))
        let six = BigFloat::from_word(6, 1);
        let three = BigFloat::from_word(3, p);

        let n = three.sqrt(p, rm);
        let n = n.reciprocal(p, rm);
        let n = n.atan(p, rm, &mut cc);
        let mut pi = six.mul(&n, p, rm);

        // Reduce precision to 1024
        pi.set_precision(1024, rm).expect("Precision updated");

        // Use library's constant for verifying the result
        let pi_lib = cc.pi(1024, rm).unwrap().into();

        // Compare computed constant with library's constant
        assert_eq!(pi.cmp(&pi_lib), Some(0));

        // Print using decimal radix.
        //println!("{}", pi);
    }
}
