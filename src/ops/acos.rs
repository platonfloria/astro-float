//! Arccosine.

use crate::num::BigFloatNumber;
use crate::defs::RoundingMode;
use crate::defs::Error;
use crate::ops::consts::Consts;


impl BigFloatNumber {

    /// Computes the arccosine of a number. The result is rounded using the rounding mode `rm`. 
    /// This function requires constants cache `cc` for computing the result.
    /// 
    /// ## Errors
    /// 
    ///  - InvalidArgument: argument is greater than 1 or smaller than -1.
    ///  - MemoryAllocation: failed to allocate memory.
    pub fn acos(&self, rm: RoundingMode, cc: &mut Consts) -> Result<Self, Error> {

        let mut x = self.clone()?;
        x.set_precision(self.get_mantissa_max_bit_len() + 1, RoundingMode::None)?;

        let mut pi = cc.pi(self.get_mantissa_max_bit_len() + 1, RoundingMode::None)?;

        pi.set_exponent(pi.get_exponent() - 1);

        let mut ret = x.asin(RoundingMode::None, cc)?;

        ret = pi.sub(&ret, RoundingMode::None)?;

        ret.set_precision(self.get_mantissa_max_bit_len(), rm)?;

        Ok(ret)
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_arccosine() {

        let mut cc = Consts::new().unwrap();

        let rm = RoundingMode::ToEven;
        let mut n1 = BigFloatNumber::from_word(4294967295,64).unwrap();
        n1.set_exponent(0);
        //println!("{}", n1.format(crate::Radix::Dec, RoundingMode::None).unwrap());
        let _n2 = n1.acos(rm, &mut cc).unwrap();
        //println!("{:?}", n2.format(crate::Radix::Dec, rm).unwrap());
    }

    #[ignore]
    #[test]
    #[cfg(feature="std")]
    fn arccosine_perf() {

        let mut cc = Consts::new().unwrap();

        let mut n = vec![];
        for _ in 0..10000 {
            n.push(BigFloatNumber::random_normal(133, -5, 5).unwrap());
        }

        for _ in 0..5 {
            let start_time = std::time::Instant::now();
            for ni in n.iter() {
                let _f = ni.acos(RoundingMode::ToEven, &mut cc).unwrap();
            }
            let time = start_time.elapsed();
            println!("{}", time.as_millis());
        }
    }

}