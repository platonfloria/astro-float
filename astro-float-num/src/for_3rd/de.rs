//! Deserialization of BigFloat.

use core::fmt::Formatter;

use crate::num::BigFloatNumber;
use crate::{BigFloat, Radix, RoundingMode};
use serde::de::Error;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};

#[cfg(not(feature = "std"))]
use {alloc::format, alloc::string::String};

pub struct BigFloatVisitor {}

impl<'de> Deserialize<'de> for BigFloat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(BigFloatVisitor {})
    }
}

impl<'de> Visitor<'de> for BigFloatVisitor {
    type Value = BigFloat;

    fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
        write!(formatter, "except `String`, `Number`, `Bytes`")
    }

    fn visit_u64<E: Error>(self, v: u64) -> Result<Self::Value, E> {
        match BigFloatNumber::from_usize(v as usize) {
            Ok(o) => Ok(o.into()),
            Err(e) => Err(Error::custom(format!("{e:?}"))),
        }
    }

    fn visit_f32<E: Error>(self, v: f32) -> Result<Self::Value, E> {
        match BigFloatNumber::from_f32(64, v) {
            Ok(o) => Ok(o.into()),
            Err(e) => Err(Error::custom(format!("{e:?}"))),
        }
    }

    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        match BigFloatNumber::from_f64(64, v) {
            Ok(o) => Ok(o.into()),
            Err(e) => Err(Error::custom(format!("{e:?}"))),
        }
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        match BigFloatNumber::parse(v, Radix::Dec, 64, RoundingMode::None) {
            Ok(o) => Ok(o.into()),
            Err(e) => Err(Error::custom(format!("{e:?}"))),
        }
    }

    fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
        self.visit_str(&v)
    }

    // lossless conversion
    // (&[Word], usize, Sign, Exponent)
    // (s * len, s    , 1   , 1       )
    // fn visit_bytes<E: Error>(self, _: &[u8]) -> Result<Self::Value, E> {
    //     todo!()
    // }
}

#[cfg(test)]
mod tests {

    use serde_json::from_str;

    use crate::BigFloat;

    #[cfg(not(feature = "std"))]
    use alloc::format;

    #[test]
    fn from_json() {
        assert_eq!("0.0", format!("{}", from_str::<BigFloat>("-0").unwrap()));
        assert_eq!("0.0", format!("{}", from_str::<BigFloat>("0.0").unwrap()));
        assert_eq!(
            "2.99999999999999988897e-1",
            format!("{}", from_str::<BigFloat>("0.3").unwrap())
        );
        assert_eq!(
            "2.99999999999999999983e-1",
            format!("{}", from_str::<BigFloat>("\"0.3\"").unwrap())
        );
    }
}
