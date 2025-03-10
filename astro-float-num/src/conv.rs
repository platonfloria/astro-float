//! Conversion utilities.

use crate::common::consts::EIGHT;
use crate::common::consts::SIXTEEN;
use crate::common::consts::TEN;
use crate::common::consts::TEN_POW_9;
use crate::common::consts::TWO;
use crate::common::util::round_p;
use crate::defs::DoubleWord;
use crate::defs::Error;
use crate::defs::Exponent;
use crate::defs::Radix;
use crate::defs::RoundingMode;
use crate::defs::Sign;
use crate::defs::Word;
use crate::defs::WORD_BIT_SIZE;
use crate::defs::WORD_MAX;
use crate::mantissa::Mantissa;
use crate::num::BigFloatNumber;
use crate::EXPONENT_MAX;
use crate::EXPONENT_MIN;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

impl BigFloatNumber {
    /// Converts an array of digits in radix `rdx` to BigFloatNumber with precision `p`.
    /// `digits` represents mantissa and is interpreted as a number smaller than 1 and greater or equal to 1/`rdx`.
    /// The first element in `digits` is the most significant digit.
    /// `e` is the exponent part of the number, such that the number can be represented as `digits` * `rdx` ^ `e`.
    /// Precision is rounded upwards to the word size.
    ///
    /// ## Errors
    ///
    ///  - MemoryAllocation: failed to allocate memory for mantissa.
    ///  - ExponentOverflow: the resulting exponent becomes greater than the maximum allowed value for the exponent.
    ///  - InvalidArgument: the precision is incorrect, or `digits` contains unacceptable digits for given radix,
    /// or when `e` is less than EXPONENT_MIN or greater than EXPONENT_MAX.
    pub fn convert_from_radix(
        sign: Sign,
        digits: &[u8],
        e: Exponent,
        rdx: Radix,
        p: usize,
        rm: RoundingMode,
    ) -> Result<Self, Error> {
        let p = round_p(p);
        Self::p_assertion(p)?;

        if p == 0 {
            return Self::new(0);
        }

        if e < EXPONENT_MIN || e > EXPONENT_MAX {
            return Err(Error::InvalidArgument);
        }

        match rdx {
            Radix::Bin => Self::conv_from_binary(sign, digits, e, p, rm),
            Radix::Oct => Self::conv_from_commensurable(sign, digits, e, 3, p, rm),
            Radix::Dec => Self::conv_from_num_dec(sign, digits, e, p, rm),
            Radix::Hex => Self::conv_from_commensurable(sign, digits, e, 4, p, rm),
        }
    }

    fn conv_from_binary(
        sign: Sign,
        digits: &[u8],
        e: Exponent,
        p: usize,
        rm: RoundingMode,
    ) -> Result<Self, Error> {
        if digits.is_empty() {
            return Self::new(p);
        }

        let mut mantissa = Mantissa::new(digits.len())?;
        let mut d = 0;
        let mut shift = digits.len() % WORD_BIT_SIZE;
        if shift != 0 {
            shift = WORD_BIT_SIZE - shift;
        }

        let mut dst = mantissa.digits_mut().iter_mut();

        for v in digits.iter().rev() {
            if *v > 1 {
                return Err(Error::InvalidArgument);
            }

            d |= (*v as Word) << shift;
            shift += 1;

            if shift == WORD_BIT_SIZE {
                *dst.next().unwrap() = d; // mantissa has precision of digits.len()
                shift = 0;
                d = 0;
            }
        }

        if shift > 0 {
            *dst.next().unwrap() = d; // mantissa has precision of digits.len()
        }

        mantissa.update_bit_len();

        let mut ret = BigFloatNumber::from_raw_unchecked(mantissa, sign, e, false);

        ret.set_precision(p, rm)?;

        Ok(ret)
    }

    // radix is power of 2.
    fn conv_from_commensurable(
        sign: Sign,
        digits: &[u8],
        e: Exponent,
        shift: usize,
        p: usize,
        rm: RoundingMode,
    ) -> Result<Self, Error> {
        let significant_bit = 1 << (shift - 1);
        let base = 1 << shift;

        // exponent shift
        let mut e_shift = 0;
        let mut zeroes = 0;
        let mut first_shift = 0;
        for v in digits {
            let mut v = *v;

            if v == 0 {
                e_shift -= shift as isize;
                zeroes += 1;
            } else {
                if v >= base {
                    return Err(Error::InvalidArgument);
                }

                while v & significant_bit == 0 {
                    e_shift -= 1;
                    v <<= 1;
                    first_shift += 1;
                }
                break;
            }
        }

        if zeroes == digits.len() {
            // mantissa is zero
            let m = Mantissa::new(p)?;

            Ok(BigFloatNumber::from_raw_unchecked(m, sign, 0, false))
        } else {
            let mut m = Mantissa::new((digits.len() - zeroes) * shift + WORD_BIT_SIZE)?;

            // exponent
            let e = e as isize * shift as isize + e_shift;

            if e > EXPONENT_MAX as isize {
                return Err(Error::ExponentOverflow(sign));
            }

            // fill mantissa
            let mut filled = shift - first_shift;
            let mut dst = m.digits_mut().iter_mut().rev();
            let mut d = 0;

            'outer: for &v in digits.iter().skip(zeroes) {
                if v >= base {
                    return Err(Error::InvalidArgument);
                }

                if filled <= WORD_BIT_SIZE {
                    d |= (v as Word) << (WORD_BIT_SIZE - filled);
                } else {
                    d |= (v as Word) >> (filled - WORD_BIT_SIZE);

                    if let Some(w) = dst.next() {
                        *w = d;
                        filled -= WORD_BIT_SIZE;
                        d = if filled > 0 { (v as Word) << (WORD_BIT_SIZE - filled) } else { 0 };
                    } else {
                        break 'outer;
                    }
                }

                filled += shift;
            }

            if d > 0 {
                if let Some(w) = dst.next() {
                    *w = d;
                }
            }

            m.set_bit_len(m.max_bit_len());

            let mut ret = if e < EXPONENT_MIN as isize {
                let mut num = BigFloatNumber::from_raw_unchecked(m, sign, EXPONENT_MIN, false);

                if p + WORD_BIT_SIZE > num.mantissa_max_bit_len() {
                    num.set_precision(p + WORD_BIT_SIZE, RoundingMode::None)?;
                }

                num.subnormalize(e, RoundingMode::None);

                if num.inexact() {
                    num.mantissa_mut().digits_mut()[0] |= 1; // sticky for correct rounding when calling set_precision()
                }

                num
            } else {
                BigFloatNumber::from_raw_unchecked(m, sign, e as Exponent, false)
            };

            ret.set_precision(p, rm)?;

            Ok(ret)
        }
    }

    fn conv_from_num_dec(
        sign: Sign,
        digits: &[u8],
        e: Exponent,
        p: usize,
        rm: RoundingMode,
    ) -> Result<Self, Error> {
        // mantissa part
        let leadzeroes = digits.iter().take_while(|&&x| x == 0).count();

        let pf = round_p(
            (((digits.len() - leadzeroes) as u64 * 3321928095 / 1000000000) as usize).max(p)
                + WORD_BIT_SIZE,
        );

        let mut f = Self::new(pf)?;

        // TODO: divide and conquer can be used to build the mantissa.
        let mut word: Word = 0;
        let mut i = 0;
        for &d in digits.iter().skip(leadzeroes) {
            if d > 9 {
                return Err(Error::InvalidArgument);
            }

            word *= 10;
            word += d as Word;

            i += 1;
            if i == 9 {
                i = 0;

                let d2 = Self::from_word(word, 1)?;
                f = f.mul(&TEN_POW_9, pf, RoundingMode::None)?;
                f = f.add(&d2, pf, RoundingMode::None)?;

                word = 0;
            }
        }

        if i > 0 {
            let mut ten_pow = 10;
            i -= 1;
            while i > 0 {
                ten_pow *= 10;
                i -= 1;
            }
            let ten_pow = Self::from_word(ten_pow, 1)?;
            let d2 = Self::from_word(word, 1)?;
            f = f.mul(&ten_pow, pf, RoundingMode::None)?;
            f = f.add(&d2, pf, RoundingMode::None)?;
        }

        // exponent part
        let n = e as isize - digits.len() as isize;

        let nmax = (EXPONENT_MAX as u64 * 301029995 / 1000000000) as usize;

        let ten = Self::from_word(10, 4)?;

        let mut nabs = n.unsigned_abs();
        if nabs > nmax {
            let fpnmax = ten.powi(nmax, pf, RoundingMode::None)?;

            while nabs > nmax {
                f = if n < 0 {
                    f.div(&fpnmax, pf, RoundingMode::None)
                } else {
                    f.mul(&fpnmax, pf, RoundingMode::None)
                }?;
                nabs -= nmax;
            }
        };

        if nabs > 0 {
            let fpn = ten.powi(nabs, pf.max(p) + WORD_BIT_SIZE, RoundingMode::None)?;

            f = if n < 0 {
                f.div(&fpn, pf, RoundingMode::None)
            } else {
                f.mul(&fpn, pf, RoundingMode::None)
            }?;
        }

        f.set_sign(sign);
        f.set_precision(p, rm)?;

        Ok(f)
    }

    /// Converts `self` to radix `rdx` using rounding mode `rm`.
    /// The function returns sign, mantissa digits in radix `rdx`, and exponent such that the converted number
    /// can be represented as `mantissa digits` * `rdx` ^ `exponent`.
    /// The first element in the mantissa is the most significant digit.
    ///
    /// ## Errors
    ///
    ///  - MemoryAllocation: failed to allocate memory for mantissa.
    ///  - ExponentOverflow: the resulting exponent becomes greater than the maximum allowed value for the exponent.
    pub fn convert_to_radix(
        &self,
        rdx: Radix,
        rm: RoundingMode,
    ) -> Result<(Sign, Vec<u8>, Exponent), Error> {
        match rdx {
            Radix::Bin => self.conv_to_binary(),
            Radix::Oct => self.conv_to_commensurable(3),
            Radix::Dec => self.conv_to_dec(rm),
            Radix::Hex => self.conv_to_commensurable(4),
        }
    }

    fn conv_to_dec(&self, rm: RoundingMode) -> Result<(Sign, Vec<u8>, Exponent), Error> {
        // input: rdx = 10, self = m*2^e, 0.5 <= m < 1,
        // let self = m*2^e * rdx^n / rdx^n, where n = floor(e * log_rdx(2))
        // let f = m / rdx^n,
        // then resulting number is F = f * rdx^n

        let n = (self.exponent().unsigned_abs() as u64 * 301029996 / 1000000000) as usize;
        let l = (self.mantissa_max_bit_len() as u64 * 301029996 / 1000000000 + 1) as usize;

        let (digits, e_shift) = if n == 0 {
            self.conv_mantissa(l, Radix::Dec, rm)
        } else {
            let p_w = self.mantissa_max_bit_len() + WORD_BIT_SIZE;

            let rdx = Self::number_for_radix(Radix::Dec)?;

            let f = if n >= 646456993 {
                // avoid powi overflow

                let d = rdx.powi(n - 1, p_w, RoundingMode::None)?;

                if self.exponent() < 0 {
                    self.mul(&d, self.mantissa_max_bit_len(), RoundingMode::None)?
                        .mul(rdx, self.mantissa_max_bit_len(), RoundingMode::None)
                } else {
                    self.div(&d, self.mantissa_max_bit_len(), RoundingMode::None)?
                        .div(rdx, self.mantissa_max_bit_len(), RoundingMode::None)
                }
            } else {
                let d = rdx.powi(n, p_w, RoundingMode::None)?;

                if self.exponent() < 0 {
                    self.mul(&d, self.mantissa_max_bit_len(), RoundingMode::None)
                } else {
                    self.div(&d, self.mantissa_max_bit_len(), RoundingMode::None)
                }
            }?;

            f.conv_mantissa(l, Radix::Dec, rm)
        }?;

        let e = (n as Exponent) * self.exponent().signum() + e_shift;

        Ok((self.sign(), digits, e))
    }

    /// Conversion for radixes of power of 2.
    fn conv_to_commensurable(&self, shift: usize) -> Result<(Sign, Vec<u8>, Exponent), Error> {
        let mut e = self.exponent();
        let mut e_shift = e.unsigned_abs() as usize % shift;
        e /= shift as Exponent;
        if e_shift != 0 && self.exponent() > 0 {
            e_shift = shift - e_shift;
            e += 1;
        }

        let mut ret = Vec::new();

        let mask = (WORD_MAX >> (WORD_BIT_SIZE - shift)) as DoubleWord;
        let mut iter = self.mantissa().digits().iter().rev();

        let mut done = WORD_BIT_SIZE - shift + e_shift;
        let mut d = *iter.next().unwrap() as DoubleWord; // iter is never empty.

        loop {
            let digit = ((d >> done) & mask) as u8;

            ret.push(digit);

            if done < shift {
                d <<= WORD_BIT_SIZE;

                if let Some(v) = iter.next() {
                    d |= *v as DoubleWord;
                } else {
                    break;
                }

                done += WORD_BIT_SIZE;
            }

            done -= shift;
        }

        if done > 0 {
            if done < shift {
                done += WORD_BIT_SIZE - shift;
            }

            let digit = ((d >> done) & mask) as u8;
            ret.push(digit);
        }

        Ok((self.sign(), ret, e))
    }

    fn conv_to_binary(&self) -> Result<(Sign, Vec<u8>, Exponent), Error> {
        let mut ret = Vec::new();

        for v in self.mantissa().digits().iter().rev() {
            for i in (0..WORD_BIT_SIZE).rev() {
                ret.push(((v >> i) & 1) as u8);
            }
        }

        Ok((self.sign(), ret, self.exponent()))
    }

    fn conv_mantissa(
        &self,
        l: usize,
        rdx: Radix,
        rm: RoundingMode,
    ) -> Result<(Vec<u8>, Exponent), Error> {
        let mut ret = Vec::new();
        let mut e_shift = 0;

        if self.is_zero() {
            ret.try_reserve_exact(1)?;
            ret.push(0);
        } else {
            ret.try_reserve_exact(3 + l)?;

            let mut r = self.clone()?;
            r.set_sign(Sign::Pos);
            r.set_precision(r.mantissa_max_bit_len() + 4, RoundingMode::None)?;

            let rdx_num = Self::number_for_radix(rdx)?;
            let rdx_word = Self::word_for_radix(rdx);

            let mut word;

            let d = r.mul(rdx_num, r.mantissa_max_bit_len(), RoundingMode::None)?;
            r = d.fract()?;
            word = d.int_as_word();
            if word == 0 {
                e_shift = -1;
                let d = r.mul(rdx_num, r.mantissa_max_bit_len(), RoundingMode::None)?;
                r = d.fract()?;
                word = d.int_as_word();
            } else if word >= rdx_word {
                e_shift = 1;

                ret.push((word / rdx_word) as u8);
                ret.push((word % rdx_word) as u8);

                let d = r.mul(rdx_num, r.mantissa_max_bit_len(), RoundingMode::None)?;
                r = d.fract()?;
                word = d.int_as_word();
            }

            for _ in 0..l {
                ret.push(word as u8);

                let d = r.mul(rdx_num, r.mantissa_max_bit_len(), RoundingMode::None)?;
                r = d.fract()?;
                word = d.int_as_word();
            }

            if !r.round(0, rm)?.is_zero() {
                word += 1;

                if word == rdx_word {
                    ret.push(0);

                    let mut i = ret.len() - 2;
                    while i > 0 && ret[i] + 1 == rdx_word as u8 {
                        ret[i] = 0;
                        i -= 1;
                    }
                    ret[i] += 1;
                } else {
                    ret.push(word as u8);
                }
            } else {
                ret.push(word as u8);
            }
        }

        // strip zeroes
        let nzeroes = ret.iter().rev().take_while(|x| **x == 0).count();
        ret.truncate(ret.len() - nzeroes);

        Ok((ret, e_shift))
    }

    fn word_for_radix(rdx: Radix) -> Word {
        match rdx {
            Radix::Bin => 2,
            Radix::Oct => 8,
            Radix::Dec => 10,
            Radix::Hex => 16,
        }
    }

    fn number_for_radix(rdx: Radix) -> Result<&'static Self, Error> {
        Ok(match rdx {
            Radix::Bin => &TWO,
            Radix::Oct => &EIGHT,
            Radix::Dec => &TEN,
            Radix::Hex => &SIXTEEN,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::common::consts::ONE;
    use crate::common::util::random_subnormal;
    use crate::defs::{Sign, EXPONENT_MAX, EXPONENT_MIN};
    use rand::random;

    #[test]
    fn test_conv() {
        // basic tests
        let n = BigFloatNumber::from_f64(64, 0.031256789f64).unwrap();

        let (s, m, e) = n.convert_to_radix(Radix::Bin, RoundingMode::None).unwrap();

        assert_eq!(
            m,
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0,
                1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(s, Sign::Pos);
        assert_eq!(e, -4);

        let g = BigFloatNumber::convert_from_radix(s, &m, e, Radix::Bin, 160, RoundingMode::ToEven)
            .unwrap();
        let f = g.to_f64();

        assert_eq!(f, 0.031256789f64);

        let n = BigFloatNumber::from_f64(64, 0.00012345678f64).unwrap();

        let (s, m, e) = n.convert_to_radix(Radix::Dec, RoundingMode::None).unwrap();

        assert_eq!(s, Sign::Pos);
        assert_eq!(
            m,
            [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 4, 2]
        );
        assert_eq!(e, -3);

        let g = BigFloatNumber::convert_from_radix(
            Sign::Neg,
            &[1, 2, 3, 4, 5, 6, 7, 0],
            3,
            Radix::Oct,
            64,
            RoundingMode::None,
        )
        .unwrap();
        let n = BigFloatNumber::from_f64(64, -83.591552734375).unwrap();
        assert_eq!(n.cmp(&g), 0);

        #[cfg(target_arch = "x86")]
        {
            let n = BigFloatNumber::from_raw_parts(
                &[2576980377, 2576980377, 2576980377],
                96,
                Sign::Pos,
                -1,
                false,
            )
            .unwrap();
            let (s, m, e) = n.convert_to_radix(Radix::Oct, RoundingMode::None).unwrap();
            let g =
                BigFloatNumber::convert_from_radix(s, &m, e, Radix::Oct, 160, RoundingMode::ToEven)
                    .unwrap();

            assert!(n.cmp(&g) == 0);

            let n = BigFloatNumber::from_raw_parts(
                &[2576980377, 2576980377, 2576980377],
                96,
                Sign::Pos,
                -0,
                false,
            )
            .unwrap();
            let (s, m, e) = n.convert_to_radix(Radix::Dec, RoundingMode::None).unwrap();

            assert_eq!(
                m,
                [
                    5, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                    9, 9, 9, 2
                ]
            );
            assert!(s == Sign::Pos);
            assert!(e == 0);

            let g =
                BigFloatNumber::convert_from_radix(s, &m, e, Radix::Dec, 96, RoundingMode::ToEven)
                    .unwrap();
            assert!(g.cmp(&n) == 0);
        }

        #[cfg(not(target_arch = "x86"))]
        {
            let n = BigFloatNumber::from_raw_parts(
                &[0x9999999999999999, 0x9999999999999999, 0x9999999999999999],
                192,
                Sign::Pos,
                -1,
                false,
            )
            .unwrap();
            let (s, m, e) = n.convert_to_radix(Radix::Oct, RoundingMode::None).unwrap();
            let g =
                BigFloatNumber::convert_from_radix(s, &m, e, Radix::Oct, 192, RoundingMode::ToEven)
                    .unwrap();

            assert!(n.cmp(&g) == 0);

            let n = BigFloatNumber::from_raw_parts(
                &[0x9999999999999999, 0x9999999999999999, 0x9999999999999999],
                192,
                Sign::Pos,
                -0,
                false,
            )
            .unwrap();
            let (s, m, e) = n
                .convert_to_radix(Radix::Dec, RoundingMode::ToEven)
                .unwrap();

            assert_eq!(
                m,
                [
                    5, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                    9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
                    9, 9, 9, 9, 9, 9
                ]
            );
            assert_eq!(s, Sign::Pos);
            assert_eq!(e, 0);

            let g =
                BigFloatNumber::convert_from_radix(s, &m, e, Radix::Dec, 192, RoundingMode::ToEven)
                    .unwrap();
            //println!("{:?}", g);
            //println!("{:?}", n);
            assert!(g.cmp(&n) == 0);
        }

        /* let n = BigFloatNumber::from_words(&[10946118985158034780, 0, 0], Sign::Pos, -2147483648).unwrap();
        let (s, m , e) = (Sign::Pos, vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 7, 5, 1, 6, 10, 5, 0, 10, 12, 6, 9, 15, 11, 9, 15, 13, 13, 10, 7, 7, 10, 10, 3, 10, 5, 13, 12, 1, 7, 11, 2, 0, 3, 0, 13, 8, 6, 4, 4, 2, 7, 4, 14, 4, 15, 2, 12, 1, 8, 15, 13, 2, 6, 13, 1, 3, 10, 12, 2, 1, 10, 1, 1, 3, 9, 2, 10, 12, 9, 9, 8, 8, 10, 9, 11, 4, 13, 10, 3, 7, 5, 10, 4, 5, 7, 3, 15, 5, 8, 7, 12, 12, 8, 14, 6, 2, 6, 7, 9, 9, 5, 0, 13, 12, 12, 10, 15, 9, 0, 0, 3, 14, 15, 1, 0, 2, 1, 10, 15, 12, 11, 1, 0, 11, 9, 0, 12, 15, 3, 7, 2, 15, 13, 7, 6, 2, 10, 8, 11, 5, 6, 8, 8, 10, 9, 12, 9, 1, 4, 8, 12, 3, 5, 4, 3, 12, 7, 5, 3, 13, 11, 11, 14, 8, 8, 14, 14, 4, 11, 5, 6, 0, 2, 1, 4, 2, 0, 10, 0, 6, 9, 0, 7, 12, 14, 3, 10, 11, 1, 8, 15, 11, 9, 3, 6, 1, 3, 15, 11, 13, 10, 6, 5, 1, 7, 11, 12, 13, 3, 5, 0, 14, 9, 9, 1, 5, 6, 1, 15, 13, 6, 9, 10, 0, 8, 1, 7, 6, 2, 10, 7, 6, 1, 7, 12, 5, 8, 4, 4, 6, 1, 3, 12, 2, 5, 0, 10, 7, 12, 5, 6, 10, 3, 5, 14, 5, 1, 9, 7, 5, 11, 13, 0, 15, 12, 2, 11, 8, 14, 4, 11, 1, 4, 4, 14, 5, 7, 12, 11, 8, 10, 8, 13, 8, 13, 3, 12, 1, 1, 7, 4, 9, 3, 2, 15, 12, 14, 2, 8, 8, 15, 2, 4, 9, 3, 9, 11, 0, 9, 7, 7, 6, 7, 14, 14, 4, 1, 2, 4, 5, 10, 13, 13, 12, 7, 5, 13, 5, 1, 1, 0, 1, 15, 2, 7, 1, 8, 7, 4, 4, 11, 4, 4, 0, 2, 12, 3, 15, 6, 15, 0, 11, 2, 6, 2, 15, 14, 5, 4, 14, 14, 1, 7, 10, 9, 3, 4, 7, 2, 2, 9, 8, 1, 13, 3, 4, 6, 3, 3, 10, 5, 5, 13, 8, 15, 8, 10, 5, 2, 15, 13, 13, 3, 11, 8, 1, 10, 8, 0, 1, 0, 14, 1, 11, 10, 10, 10, 0, 13, 9, 8, 6, 13, 9, 0, 0, 15, 5, 4, 12, 5, 15, 11, 9, 5, 5, 10, 0, 2, 3, 12, 11, 4, 14, 13, 9, 13, 0, 6, 0, 9, 12, 3, 5, 13, 5, 3, 8, 8, 6, 5, 2, 5, 9, 13, 2, 13, 12, 4, 1, 13, 13, 9], -536870912);

        let g = BigFloatNumber::convert_from_radix(s, &m, e, Radix::Hex, 1920, RoundingMode::ToEven).unwrap();
        println!("{:?}", g);
        return; */

        // random normal values
        let mut eps = ONE.clone().unwrap();
        let p_rng = 32;

        for _ in 0..1000 {
            let p1 = (random::<usize>() % p_rng + 1) * WORD_BIT_SIZE;
            let p2 = (random::<usize>() % p_rng + 1) * WORD_BIT_SIZE;
            let p = p1.min(p2);

            let mut n =
                BigFloatNumber::random_normal(p1, EXPONENT_MIN + p1 as Exponent, EXPONENT_MAX)
                    .unwrap();
            let rdx = random_radix();

            let (s1, m1, e1) = n.convert_to_radix(rdx, RoundingMode::ToEven).unwrap();
            let mut g =
                BigFloatNumber::convert_from_radix(s1, &m1, e1, rdx, p2, RoundingMode::ToEven)
                    .unwrap();

            //println!("\n{:?}", rdx);
            //println!("{:?} {:?} {}", s1, m1, e1);
            //println!("{:?}\n{:?}", n, g);

            if rdx == Radix::Dec {
                eps.set_exponent(n.exponent() - p as Exponent + 3);
                assert!(
                    n.sub(&g, p, RoundingMode::None)
                        .unwrap()
                        .abs()
                        .unwrap()
                        .cmp(&eps)
                        <= 0
                );
            } else {
                if p2 < p1 {
                    n.set_precision(p, RoundingMode::ToEven).unwrap();
                } else if p2 > p1 {
                    g.set_precision(p, RoundingMode::ToEven).unwrap();
                }

                assert!(n.cmp(&g) == 0);
            }
        }

        // subnormal values
        for _ in 0..1000 {
            let p1 = (random::<usize>() % p_rng + 3) * WORD_BIT_SIZE;
            let p2 = (random::<usize>() % p_rng + 3) * WORD_BIT_SIZE;
            let p = p1.min(p2);

            let mut n = random_subnormal(p1);
            let rdx = random_radix();

            let (s1, m1, e1) = n.convert_to_radix(rdx, RoundingMode::ToEven).unwrap();

            //println!("\n{:?}", rdx);
            //println!("{:?} {:?} {}", s1, m1, e1);

            let mut g =
                BigFloatNumber::convert_from_radix(s1, &m1, e1, rdx, p2, RoundingMode::ToEven)
                    .unwrap();

            //println!("\n{:?}", rdx);
            //println!("{:?} {:?} {}", s1, m1, e1);
            //println!("{:?}\n{:?}", n, g);

            if rdx == Radix::Dec {
                let mut eps = BigFloatNumber::min_positive(p).unwrap();
                eps.set_exponent(eps.exponent() + 1);

                assert!(
                    n.sub(&g, p, RoundingMode::None)
                        .unwrap()
                        .abs()
                        .unwrap()
                        .cmp(&eps)
                        <= 0,
                    "{:?} {:?}",
                    n,
                    g
                );
            } else {
                if p2 < p1 {
                    n.set_precision(p, RoundingMode::ToEven).unwrap();
                } else if p2 > p1 {
                    g.set_precision(p, RoundingMode::ToEven).unwrap();
                }

                assert!(n.cmp(&g) == 0, "{:?} {:?} {:?}", rdx, n, g);
            }
        }

        // MIN, MAX, min_subnormal
        let p1 = (random::<usize>() % p_rng + 1) * WORD_BIT_SIZE;
        let p2 = (random::<usize>() % p_rng + 1) * WORD_BIT_SIZE;
        let p = p1.min(p2);

        for rdx in [Radix::Bin, Radix::Oct, Radix::Dec, Radix::Hex] {
            // min, max
            // for p2 < p1 rounding will cause overflow, for p2 >= p1 no rounding is needed.
            let rm = RoundingMode::None;
            for mut n in
                [BigFloatNumber::max_value(p1).unwrap(), BigFloatNumber::min_value(p1).unwrap()]
            {
                //println!("\n{:?} {} {}", rdx, p1, p2);
                //println!("{:?}", n);

                let (s1, m1, e1) = n.convert_to_radix(rdx, rm).unwrap();

                //println!("{:?} {:?} {}", s1, m1, e1);

                let mut g = BigFloatNumber::convert_from_radix(s1, &m1, e1, rdx, p2, rm).unwrap();

                //println!("{:?}", g);

                if rdx == Radix::Dec {
                    eps.set_exponent(n.exponent() - p as Exponent + 3);
                    assert!(n.sub(&g, p, rm).unwrap().abs().unwrap().cmp(&eps) <= 0);
                } else {
                    if p2 < p1 {
                        n.set_precision(p, rm).unwrap();
                    } else if p2 > p1 {
                        g.set_precision(p, rm).unwrap();
                    }

                    assert!(n.cmp(&g) == 0);
                }
            }

            // min subnormal
            let rm = RoundingMode::ToEven;
            let mut n = BigFloatNumber::min_positive(p1).unwrap();
            //println!("\n{:?} {} {}", rdx, p1, p2);
            //println!("{:?}", n);
            let (s1, m1, e1) = n.convert_to_radix(rdx, rm).unwrap();

            //println!("{:?} {:?} {}", s1, m1, e1);

            let mut g = BigFloatNumber::convert_from_radix(s1, &m1, e1, rdx, p2, rm).unwrap();
            //println!("{:?}", g);

            if rdx == Radix::Dec {
                let mut eps = BigFloatNumber::min_positive(p).unwrap();
                eps.set_exponent(eps.exponent() + 1);
                assert!(n.sub(&g, p, rm).unwrap().abs().unwrap().cmp(&eps) <= 0);
            } else {
                if p2 < p1 {
                    n.set_precision(p, rm).unwrap();
                } else if p2 > p1 {
                    g.set_precision(p, rm).unwrap();
                }
                assert!(n.cmp(&g) == 0);
            }
        }

        // misc/invalid input
        let s1 = Sign::Pos;
        for rdx in [Radix::Bin, Radix::Oct, Radix::Dec, Radix::Hex] {
            for e1 in [123, -123, 0] {
                let m1 = [];
                assert!(BigFloatNumber::convert_from_radix(
                    s1,
                    &m1,
                    e1,
                    rdx,
                    p1,
                    RoundingMode::ToEven
                )
                .unwrap()
                .is_zero());
                let m1 = [1, rdx as u8, 0];
                assert!(
                    BigFloatNumber::convert_from_radix(s1, &m1, e1, rdx, p1, RoundingMode::ToEven)
                        .unwrap_err()
                        == Error::InvalidArgument
                );
                let m1 = [1, rdx as u8 - 1, 0];
                assert!(BigFloatNumber::convert_from_radix(
                    s1,
                    &m1,
                    e1,
                    rdx,
                    0,
                    RoundingMode::ToEven
                )
                .unwrap()
                .is_zero());
                let m1 = [0; 256];
                assert!(BigFloatNumber::convert_from_radix(
                    s1,
                    &m1,
                    e1,
                    rdx,
                    p1,
                    RoundingMode::ToEven
                )
                .unwrap()
                .is_zero());
                let m1 = [0];
                assert!(BigFloatNumber::convert_from_radix(
                    s1,
                    &m1,
                    e1,
                    rdx,
                    p1,
                    RoundingMode::ToEven
                )
                .unwrap()
                .is_zero());
            }
        }
    }

    fn random_radix() -> Radix {
        match random::<usize>() % 4 {
            0 => Radix::Bin,
            1 => Radix::Oct,
            2 => Radix::Dec,
            3 => Radix::Hex,
            _ => unreachable!(),
        }
    }
}
