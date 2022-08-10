//! Mantissa of a number.


use crate::defs::Digit;
use crate::defs::Error;
use crate::defs::DoubleDigit;
use crate::defs::DigitSigned;
use crate::defs::DIGIT_MAX;
use crate::defs::DIGIT_BASE;
use crate::defs::DIGIT_BIT_SIZE;
use crate::defs::DIGIT_SIGNIFICANT_BIT;
use crate::defs::RoundingMode;
use crate::mantissa::util::ExtendedSlice;
use crate::mantissa::util::RightShiftedSlice;
use crate::mantissa::buf::DigitBuf;
use core::mem::size_of;
use itertools::izip;


/// Mantissa representation.
#[derive(Debug)]
pub struct Mantissa {
    pub(super) m: DigitBuf,
    pub(super) n: usize,   // number of bits, 0 is for number 0
}

impl Mantissa {

    // bit lenth to length in "digits".
    #[inline]
    fn bit_len_to_digit_len(p: usize) -> usize {
        (p + DIGIT_BIT_SIZE - 1) / size_of::<Digit>() / 8
    }

    // reserve a buffer for mantissa.
    fn reserve_new(sz: usize) -> Result<DigitBuf, Error> {
        DigitBuf::new(sz)
    }

    /// New mantissa with length of at least `p` bits filled with zeroes.
    pub fn new(p: usize) -> Result<Self, Error> {
        let mut m = Self::reserve_new(Self::bit_len_to_digit_len(p))?;
        m.fill(0);
        Ok(Mantissa {
            m,
            n: 0,
        })
    }

    /// New mantissa with length of at least `p` bits filled with 1.
    pub fn oned_mantissa(p: usize) -> Result<Self, Error> {
        let mut m = Self::reserve_new(Self::bit_len_to_digit_len(p))?;
        let n = DIGIT_BIT_SIZE*m.len();
        m.fill(DIGIT_MAX);
        Ok(Mantissa {
            m,
            n,
        })
    }

    /// New mantissa with length of at least `p` bits for the value of minimum positive value.
    pub fn min(p: usize) -> Result<Self, Error> {
        let mut m = Self::reserve_new(Self::bit_len_to_digit_len(p))?;
        m.fill(0);
        m[0] = 1;
        Ok(Mantissa {
            m,
            n: 1,
        })
    }

    /// New mantissa with length of at least `p` bits for the value of 1.
    pub fn one(p: usize) -> Result<Self, Error> {
        let mut m = Self::reserve_new(Self::bit_len_to_digit_len(p))?;
        m.fill(0);
        let l = m.len();
        m[l - 1] = (DIGIT_BASE >> 1) as Digit;
        let n = DIGIT_BIT_SIZE*m.len();
        Ok(Mantissa {
            m,
            n,
        })
    }

    /// New mantissa with length of at least `p` bits for the value of 10.
    pub fn ten(p: usize) -> Result<Self, Error> {
        let mut m = Self::reserve_new(Self::bit_len_to_digit_len(p))?;
        m.fill(0);
        let l = m.len();
        m[l - 1] = (DIGIT_BASE >> 1 | DIGIT_BASE >> 3) as Digit;
        let n = DIGIT_BIT_SIZE*m.len();
        Ok(Mantissa {
            m,
            n,
        })
    }

    /// Return true if mantissa represents zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.n == 0
    }

    /// Shift right by n bits.
    pub fn shift_right(&mut self, n: usize) {
        let idx = n / DIGIT_BIT_SIZE;
        let shift = n % DIGIT_BIT_SIZE;
        let mut d;
        if idx >= self.len() {
            self.m.fill(0);
        } else if shift > 0 {
            for i in 0..self.len() {
                d = 0;
                if idx + i + 1 < self.len() {
                    d = self.m[idx + i + 1] as DoubleDigit;
                    d <<= DIGIT_BIT_SIZE;
                }
                if i + idx < self.len() {
                    d |= self.m[idx + i] as DoubleDigit;    
                }
                d >>= shift;
                self.m[i] = d as Digit;
            }
        } else if idx > 0 {
            let src = self.m[idx..].as_ptr();
            let dst = self.m.as_mut_ptr();
            let cnt = self.len() - idx;
            unsafe {
                core::intrinsics::copy(src, dst, cnt);
            };
            self.m[cnt..].fill(0);
        }
    }

    /// Shift self left by n digits.
    #[inline]
    pub fn shift_left(&mut self, n: usize) {
        Self::shift_left_m(&mut self.m, n)
    }

    // Shift m left by n digits.
    fn shift_left_m(m: &mut [Digit], n: usize) {
        let idx = n / DIGIT_BIT_SIZE;
        let shift = n % DIGIT_BIT_SIZE;
        let mut d;
        if idx >= m.len() {
            m.fill(0);
        } else if shift > 0 {
            let mut i = m.len() - 1;
            loop {
                d = 0;
                if i >= idx {
                    d = m[i - idx] as DoubleDigit;
                    d <<= DIGIT_BIT_SIZE;
                }
                if i > idx {
                    d |= m[i - idx - 1] as DoubleDigit;    
                }
                d >>= DIGIT_BIT_SIZE - shift;
                m[i] = d as Digit;
                if i == 0 {
                    break;
                }
                i -= 1;
            }
        } else if idx > 0 {
            let dst = m[idx..].as_mut_ptr();
            let src = m.as_ptr();
            unsafe {
                core::intrinsics::copy(src, dst, m.len()-idx);
            };
            m[..idx].fill(0);
        }
    }

    /// Shift to the left, returns exponent shift as positive value.
    fn maximize(m: &mut [Digit]) -> usize {
        let mut shift = 0;
        let mut d = 0;
        for v in m.iter().rev() {
            d = *v;
            if d != 0 {
                break;
            }
            shift += DIGIT_BIT_SIZE;
        }
        if d != 0 {
            while DIGIT_SIGNIFICANT_BIT & d == 0 {
                d <<= 1;
                shift += 1;
            }
            Self::shift_left_m(m, shift);
            shift
        } else {
            0
        }
    }

    /// Compare to m2 and return positive if self > m2, negative if self < m2, 0 otherwise.
    pub fn abs_cmp(&self, m2: &Self) -> DigitSigned {
        let len = self.len().min(m2.len());
        for (a, b) in core::iter::zip(self.m.iter().rev(), m2.m.iter().rev()) {
            let diff = *a as DigitSigned - *b as DigitSigned;
            if diff != 0 {
                return diff;
            }
        }
        for v in &self.m[..self.m.len() - len] {
            if *v != 0 {
                return 1;
            }
        }
        for v in &m2.m[..m2.m.len() - len] {
            if *v != 0 {
                return -1;
            }
        }
        0
    }

    /// Subtracts m2 from self. m2 is supposed to be shifted right by m2_shift bits.
    pub fn abs_sub(&self, m2: &Self, m2_shift: usize, rm: RoundingMode, is_positive: bool) -> Result<(usize, Self), Error> {
        // Input is expected to be normalized.
        let mut c: DoubleDigit = 0;
        let l = self.len().max(m2.len()) + 1;
        let mut m3 = Mantissa::new(l*DIGIT_BIT_SIZE)?;
        let m1 = ExtendedSlice::new(self.m.iter(), l - self.len(), &0);
        let m2 = RightShiftedSlice::new(&m2.m, m2_shift, 0, true);
        for (a, b, d) in izip!(m1, m2, m3.m.iter_mut()) {
            let v1 = *a as DoubleDigit;
            let v2 = b as DoubleDigit;
            if v1 < v2 + c {
                *d = (v1 + DIGIT_BASE - v2 - c) as Digit;
                c = 1;
            } else {
                *d = (v1 - v2 - c) as Digit;
                c = 0;
            }
        }
        debug_assert!(c == 0);
        let shift = Self::maximize(&mut m3.m);
        debug_assert!(!m3.round_mantissa(DIGIT_BIT_SIZE, rm, is_positive));
        m3.m.trunc_to((l-1)*DIGIT_BIT_SIZE);
        m3.n = m3.max_bit_len();
        Ok((shift, m3))
    }

    /// Returns carry flag, and self + m2.
    pub fn abs_add(&self, m2: &Self, m2_shift: usize, rm: RoundingMode, is_positive: bool) -> Result<(bool, Self), Error> {
        let mut c = 0;
        let l = self.len().max(m2.len()) + 1;
        let mut m3 = Mantissa::new(l*DIGIT_BIT_SIZE)?;
        let m1 = ExtendedSlice::new(self.m.iter(), l - self.len(), &0);
        let m2 = RightShiftedSlice::new(&m2.m, m2_shift, 0, true);
        for (a, b, d) in izip!(m1, m2, m3.m.iter_mut()) {
            let mut s = c + *a as DoubleDigit + b as DoubleDigit;
            if s >= DIGIT_BASE {
                s -= DIGIT_BASE;
                c = 1;
            } else {
                c = 0;
            }
            *d = s as Digit;
        }
        if c > 0 {
            debug_assert!(!m3.round_mantissa(1 + DIGIT_BIT_SIZE, rm, is_positive));  // it is not possible that rounding overflows, and c > 0 at the same time.
            m3.shift_right(1);
            let l = m3.m.len();
            m3.m[l - 1] |= DIGIT_SIGNIFICANT_BIT;
        } else if m3.round_mantissa(DIGIT_BIT_SIZE, rm, is_positive) {
            c = 1;
        }
        m3.m.trunc_to((l-1)*DIGIT_BIT_SIZE);
        m3.n = m3.max_bit_len();
        Ok((c > 0, m3))
    }

    /// Multiply two mantissas, return result and exponent shift as positive value.
    pub fn mul(&self, m2: &Self, rm: RoundingMode, is_positive: bool) -> Result<(usize, Self), Error> {
        let (l, sm, lg) = if self.len() < m2.len() {
            (m2.len(), self, m2)
        } else {
            (self.len(), m2, self)
        };
        let l = l*DIGIT_BIT_SIZE;

        let mut m3 = Self::reserve_new(self.len() + m2.len())?;
        if Self::toom3_cost_estimate(sm.len(), lg.len()) {
            // toom-3
            m3[..sm.len()].copy_from_slice(&sm.m);
            m3[sm.len()..].fill(0);
            let sign = Self::toom3(&mut m3, &lg.m)?;
            debug_assert!(sign > 0);
        } else {
            // plain multiplication
            m3.fill(0);
            for (i, d1mi) in self.m.iter().enumerate() {
                let d1mi = *d1mi as DoubleDigit;
                if d1mi == 0 {
                    continue;
                }

                let mut k = 0;
                for (m2j, m3ij) in m2.m.iter().zip(m3[i..].iter_mut()) {
                    let m = d1mi * (*m2j as DoubleDigit) + *m3ij as DoubleDigit + k;

                    *m3ij = m as Digit;
                    k = m >> (DIGIT_BIT_SIZE);
                }
                m3[i + m2.len()] += k as Digit;
            }
        }
        // TODO: since leading digit is always >= 0x8000 (most significant bit is set),
        // then shift is always 0 or 1
        let mut shift = Self::maximize(&mut m3);
        let bit_len = m3.len()*DIGIT_BIT_SIZE;
        let mut ret = Mantissa {m: m3, n: bit_len};
        if ret.round_mantissa(bit_len - l, rm, is_positive) {
            shift += 1;
        }
        ret.m.trunc_to(l);
        ret.n = l;
        debug_assert!(shift <= 2);  // prevent exponent overflow
        Ok((shift, ret))
    }

    // Estimate cost of multiplication with toom-3. 
    // Return true if toom-3 is better than plain multiplication.
    // l1 is supposed to be smaller or equal to l2.
    fn toom3_cost_estimate(l1: usize, l2: usize) -> bool {
        if l1 < 70 && l2 < 70 {
            return false;
        }
        for (thrsh1, thrsh2) in [
            (120, 210),
            (200, 630),
            (340, 1890),
            (580, 5670),
            (900, 17010),
            (1500, 51030)]
        {
            if l2 < thrsh2 {
                return l1 >= thrsh1;
            }
        }
        let mut thrsh2 = 51030;
        let mut thrsh1 = 1500;
        while thrsh2 < usize::MAX / 3 {
            thrsh2 *= 3;
            thrsh1 *= 16;
            thrsh1 /= 10;
            if l2 < thrsh2 {
                return l1 >= thrsh1;
            }
        }
        false
    }

    /// Divide mantissa by mantissa, return result and exponent ajustment.
    pub fn div(&self, m2: &Self, rm: RoundingMode, is_positive: bool) -> Result<(usize, Self), Error> {
        // Knuth's division
        let extra_p = 2;
        let l1 = self.m.len().max(m2.m.len()) + extra_p;
        let l2 = m2.m.len();
        let mut m3 = Self::new(l1*DIGIT_BIT_SIZE)?;
        let mut c: DoubleDigit;
        let mut j: isize;
        let mut qh: DoubleDigit;
        let mut k: DoubleDigit;
        let mut rh: DoubleDigit;
        let mut buf = Self::reserve_new(l1 * 2 + l2 + 4)?;
        let (buf1, buf2) = (&mut buf).split_at_mut(l1 * 2 + 3);
        let n1: usize = 2 + l1;
        let mut n1j: usize;
        let n = l2 - 1;
        let m = l1 - 1;
        let mut e_shift = 1;

        if n == 0 {
            // division by single digit
            let d = m2.m[0] as DoubleDigit;
            rh = 0;
            let mut iter = self.m.iter().rev();
            let mut val = *iter.next().unwrap_or(&0) as DoubleDigit;
            if val < d {
                rh = val;
                e_shift = 0;
                val = *iter.next().unwrap_or(&0) as DoubleDigit;
            }

            let mut m3iter = m3.m.iter_mut().rev();
            loop {
                qh = rh * DIGIT_BASE as DoubleDigit + val;
                rh = qh % d;

                if let Some(v) = m3iter.next() {
                    *v = (qh / d) as Digit;
                } else {
                    break;
                }
                val = *iter.next().unwrap_or(&0) as DoubleDigit;
            }
        } else {
            // normalize: buf1 = d1 * d, buf2 = d2 * d
            let d = DIGIT_BASE / (m2.m[m2.len()-1] as DoubleDigit + 1); // factor d: d * m2[most significant] is close to DIGIT_MAX

            (&mut buf1[..n1+extra_p]).fill(0);
            (&mut buf1[self.len() + n1+extra_p..]).fill(0);
            (&mut buf2[m2.len()..]).fill(0);
            if d == 1 {
                buf1[n1+extra_p..(self.len() + n1+extra_p)].clone_from_slice(&self.m[..]);
                buf2[..m2.len()].clone_from_slice(&m2.m[..]);
            } else {
                Self::mul_by_digit(&self.m, d, &mut buf1[n1+extra_p..]);
                Self::mul_by_digit(&m2.m, d, buf2);
            }

            let v1 = buf2[n] as DoubleDigit;
            let v2 = buf2[n - 1] as DoubleDigit;

            j = m as isize - n as isize;
            n1j = (n1 as isize + j) as usize;
            let mut m3iter = m3.m.iter_mut().rev();
            let mut in_loop = false;
            let mut buf12;
            let mut buf11;
            let mut buf10;
            loop {
                buf12 = buf1[n1j + n + 1] as DoubleDigit;
                buf11 = buf1[n1j + n] as DoubleDigit;
                buf10 = buf1[n1j + n - 1] as DoubleDigit;

                qh = buf12 * DIGIT_BASE + buf11;
                rh = qh % v1;
                qh /= v1;

                if qh >= DIGIT_BASE || (qh * v2 > DIGIT_BASE * rh + buf10) {
                    qh -= 1;
                    rh += v1;
                    if rh < DIGIT_BASE && 
                        (qh >= DIGIT_BASE || (qh * v2 > DIGIT_BASE * rh + buf10)) {
                            qh -= 1;
                    }
                }

                // n1_j = n1_j - n2 * qh
                c = 0;
                k = 0;
                for (a, b) in buf2[..n+2].iter().zip(buf1[n1j..n1j+n+2].iter_mut()) {
                    k = *a as DoubleDigit * qh + k / DIGIT_BASE;
                    let val = k % DIGIT_BASE + c;
                    if (*b as DoubleDigit) < val {
                        *b += (DIGIT_BASE - val) as Digit;
                        c = 1;
                    } else {
                        *b -= val as Digit;
                        c = 0;
                    }
                }

                if c > 0 {
                    // compensate
                    qh -= 1;
                    c = 0;
                    for (a, b) in buf2[..n+2].iter().zip(buf1[n1j..n1j+n+2].iter_mut()) {
                        let mut val = *b as DoubleDigit;
                        val += *a as DoubleDigit + c;
                        if val >= DIGIT_BASE {
                            val -= DIGIT_BASE;
                            c = 1;
                        } else {
                            c = 0;
                        }
                        *b = val as Digit;
                    }
                    debug_assert!(c > 0);
                }

                if let Some(v) = m3iter.next() {
                    if in_loop || qh > 0 {
                        *v = qh as Digit;
                    } else {
                        e_shift = 0;
                    }
                } else {
                    break;
                }

                j -= 1;
                if n1 as isize + j < 0 {
                    break;
                } else {
                    n1j = (n1 as isize + j) as usize;
                }
                in_loop = true;
            }
        }

        let _ = Self::maximize(&mut m3.m);
        if m3.round_mantissa(extra_p*DIGIT_BIT_SIZE, rm, is_positive) {
            e_shift += 1;
        }
        m3.m.trunc_to((l1-extra_p)*DIGIT_BIT_SIZE);
        m3.n = m3.max_bit_len();
        Ok((e_shift, m3))
    }

    // Multiply d1 by digit d and put result to d3 with overflow.
    fn mul_by_digit(d1: &[Digit], d: DoubleDigit, d3: &mut [Digit]) {
        let mut m: DoubleDigit = 0;
        for (v1, v2) in d1.iter().zip(d3.iter_mut()) {
            m = *v1 as DoubleDigit * d + m / DIGIT_BASE;
            *v2 = (m % DIGIT_BASE) as Digit;
        }
        d3[d1.len()] = (m / DIGIT_BASE) as Digit;
    }

    pub fn from_u64(p: usize, mut u: u64) -> Result<(usize, Self), Error> {
        let mut m = Self::reserve_new(Self::bit_len_to_digit_len(p))?;
        let nd = m.len() - size_of::<u64>()/size_of::<Digit>();
        m[..nd].fill(0);
        for v in &mut m[nd..] {
            *v = u as Digit;
            u >>= DIGIT_BIT_SIZE;
        }
        let shift = Self::maximize(&mut m);
        let mut ret = Mantissa {
            m,
            n: 0,
        };
        ret.n = ret.max_bit_len();
        Ok((shift, ret))
    }

    pub fn to_u64(&self) -> u64 {
        let mut ret: u64 = 0;
        let nd = size_of::<u64>()/size_of::<Digit>();
        ret |= self.m[self.len() - 1] as u64;
        for i in 1..nd {
            ret <<= DIGIT_BIT_SIZE;
            ret |= if self.len() > i { self.m[self.len() - i - 1] as u64 } else { 0 };
        }
        ret
    }

    /// Returns true if `self` is subnormal.
    #[inline]
    pub fn is_subnormal(&self)-> bool {
        self.n < self.max_bit_len()
    }

    /// Shift to the left and return shift value.
    pub fn normilize(&self) -> Result<(usize, Self), Error> {
        let shift = self.max_bit_len() - self.n;
        let mut m = self.clone()?;
        if shift > 0 {
            Self::shift_left_m(&mut m.m, shift);
            m.n = self.max_bit_len();
        }
        Ok((shift, m))
    }

    /// Set n bits to 0 from the right.
    pub fn mask_bits(&mut self, mut n: usize) {
        for v in self.m.iter_mut() {
            if n >= DIGIT_BIT_SIZE {
                *v = 0;
                n -= DIGIT_BIT_SIZE;
            } else {
                let mask = DIGIT_MAX << n;
                *v &= mask;
            }
        }
    }

    /// Decompose to raw parts.
    pub fn to_raw_parts(&self) -> (&[Digit], usize) {
        (&self.m, self.n)
    }

    /// Construct from raw parts.
    pub fn from_raw_parts(m: &[Digit], n: usize) -> Result<Self, Error> {
        let mut mm = Self::reserve_new(m.len())?;
        mm.copy_from_slice(m);
        Ok(Mantissa {m: mm, n})
    }

    /// Returns true if all digits are equal to 0.
    pub fn is_all_zero(&self) -> bool {
        for v in (&self.m).iter() {
            if *v != 0 {
                return false;
            }
        }
        true
    }

    /// Decrement length by l, or set lentgh = 0, if l > length
    pub fn dec_len(&mut self, l: usize) {
        if self.n > l {
            self.n -= l;
        } else {
            self.n = 0;
        }
    }

    /// Returns length of the mantissa in digits of the mantissa's base.
    #[inline]
    pub fn len(&self) -> usize {
        self.m.len()
    }

    /// Returns maximum possible length of the mantissa in digits of the mantissa's base.
    #[inline]
    pub fn max_bit_len(&self) -> usize {
        self.len()*DIGIT_BIT_SIZE
    }

    // Round n positons, return true if exponent is to be incremented.
    pub fn round_mantissa(&mut self, n: usize, rm: RoundingMode, is_positive: bool) -> bool {
        let self_len = self.m.len();
        if n > 0 && n <= self.max_bit_len() {
            let n = n-1;
            let mut rem_zero = true;
            // anything before n'th digit becomes 0
            for v in &mut self.m[..n / DIGIT_BIT_SIZE] {
                if *v != 0 {
                    rem_zero = false;
                }
                *v = 0;
            }

            // analyze digits at n and at n+1
            // to decide if we need to add 1 or not.
            let mut c = false;
            let np1 = n + 1;
            let mut i = n / DIGIT_BIT_SIZE;
            let i1 = np1 / DIGIT_BIT_SIZE;
            let t = n % DIGIT_BIT_SIZE;
            let t2 = np1 % DIGIT_BIT_SIZE;
            let num = (self.m[i] >> t) & 1;
            if t > 0 && self.m[i] << (DIGIT_BIT_SIZE - t) as Digit != 0 {
                rem_zero = false;
            }

            let num2 = if i1 < self_len {
                (self.m[i1] >> t2) & 1
            } else {
                0
            };

            let eq1 = num == 1 && rem_zero;
            let gt1 = num == 1 && !rem_zero;
            let gte1 = num == 1;

            match rm {
                RoundingMode::Up => if gte1 && is_positive || gt1 && !is_positive {
                    // add 1
                    c = true;
                },
                RoundingMode::Down => if gt1 && is_positive || gte1 && !is_positive {
                    // add 1
                    c = true;
                },
                RoundingMode::FromZero => if gte1 {
                    // add 1
                    c = true;
                },
                RoundingMode::ToZero => if gt1 {
                    // add 1
                    c = true;
                },
                RoundingMode::ToEven => if gt1 || (eq1 && num2 & 1 != 0) {
                    // add 1
                    c = true;
                },
                RoundingMode::ToOdd => if gt1 || (eq1 && num2 & 1 == 0) {
                    // add 1
                    c = true;
                },
            };

            if c {
                // add 1 at (n+1)'th position
                if i1 > i {
                    self.m[i] = 0;
                }
                i = i1;
                if i < self_len {
                    if (self.m[i] >> t2) as DoubleDigit + 1 < (DIGIT_BASE >> t2) {
                        self.m[i] = ((self.m[i] >> t2) + 1) << t2;
                        return false;
                    } else {
                        self.m[i] = 0;
                    }
                }

                // process overflows
                i += 1;
                for v in &mut self.m[i..] {
                    if *v < DIGIT_MAX {
                        *v += 1;
                        return false;
                    } else {
                        *v = 0;
                    }
                    i += 1;
                }
                self.m[self_len - 1] = DIGIT_SIGNIFICANT_BIT;
                return true;
            } else {
                // just remove trailing digits
                let t = t + 1;
                self.m[i] = if t >= DIGIT_BIT_SIZE { 0 } else { (self.m[i] >> t) << t };
            }
        }
        false
    }

    pub fn get_most_significant_digit(&self) -> Digit {
        if self.n > 0 {
            self.m[(self.n - 1) / DIGIT_BIT_SIZE]
        } else {
            0
        }
    }

    #[cfg(feature="random")]
    /// Returns randomized mantissa with at least p bits of length.
    pub fn random_normal(p: usize) -> Result<Self, Error> {
        let mut m = Self::reserve_new(Self::bit_len_to_digit_len(p))?;
        for v in (&mut m).iter_mut() {
            *v = rand::random::<Digit>();
        }
        let mut ret = Mantissa {
            m,
            n: 0,
        };
        if !ret.is_all_zero() {
            Self::maximize(&mut ret.m);
            ret.n = DIGIT_BIT_SIZE*ret.m.len();
            ret.m[0] ^= rand::random::<Digit>();
        }
        Ok(ret)
    }

    /// Clones the mantissa.
    pub fn clone(&self) -> Result<Self, Error> {
        let mut m = Self::reserve_new(self.m.len())?;
        (&mut m).copy_from_slice(&self.m);
        Ok(Mantissa {
            m,
            n: self.n,
        })
    }

    /// Copy the contents of `m2`. 
    /// If `m2` is shorter than `self`, then the remaining digits are set to 0.
    pub fn copy_from(&mut self, m2: &Self) {
        let l1 = self.len();
        let l2 = m2.len();
        let l = l1.min(l2);
        self.m[l1-l..].copy_from_slice(&m2.m[l2-l..]);
        self.m[..l1-l].fill(0);
        self.n = m2.n;
        if l < l2 {
            self.n -= (l2 - l)*DIGIT_BIT_SIZE;
        } else if l < l1 {
            self.n += (l1 - l)*DIGIT_BIT_SIZE;
        }
    }
}


mod buf {

    
}