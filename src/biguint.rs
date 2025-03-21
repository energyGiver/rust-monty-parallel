//! A minimal BigUint implementation for demonstration purposes.
//! 이 구현은 여러 limb에 대해 올바른 left-shift, bit_length(), 및 모듈러 연산을 지원합니다.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigUint {
    pub data: Vec<u64>,
}

impl BigUint {
    /// 0을 나타내는 BigUint.
    pub fn zero() -> Self {
        BigUint { data: vec![0] }
    }
    /// 1을 나타내는 BigUint.
    pub fn one() -> Self {
        BigUint { data: vec![1] }
    }
    /// 불필요한 trailing zero limb들을 제거.
    pub fn normalize(&mut self) {
        while self.data.len() > 1 && *self.data.last().unwrap() == 0 {
            self.data.pop();
        }
    }
    /// BigUint의 실제 유효 비트 수를 반환합니다.
    pub fn bit_length(&self) -> u64 {
        if let Some(&last) = self.data.last() {
            let bits = 64 - last.leading_zeros() as u64;
            bits + 64 * ((self.data.len() - 1) as u64)
        } else {
            0
        }
    }
}

use core::ops::Shl;

impl Shl<u64> for BigUint {
    type Output = Self;
    fn shl(self, shift: u64) -> Self::Output {
        let limb_bits = 64;
        let limb_shift = (shift / limb_bits) as usize;
        let bit_shift = shift % limb_bits;

        let mut result = Vec::with_capacity(self.data.len() + limb_shift + 1);
        for _ in 0..limb_shift {
            result.push(0);
        }

        let mut carry = 0u64;
        for &d in self.data.iter() {
            let new_d = (d << bit_shift) | carry;
            result.push(new_d);
            if bit_shift == 0 {
                carry = 0;
            } else {
                carry = d >> (limb_bits - bit_shift);
            }
        }
        if carry != 0 {
            result.push(carry);
        }
        BigUint { data: result }
    }
}

use core::ops::{Mul, Rem, Sub};

impl Sub for BigUint {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        BigUint {
            data: vec![self.data[0] - rhs.data[0]],
        }
    }
}

impl Sub<&BigUint> for BigUint {
    type Output = Self;
    fn sub(self, rhs: &BigUint) -> Self::Output {
        BigUint {
            data: vec![self.data[0] - rhs.data[0]],
        }
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.data[0].partial_cmp(&other.data[0])
    }
}

/// 모듈러 연산: 각 limb마다 Horner's method를 적용하여 모듈러 연산을 수행합니다.
/// m이 한 limb (u64)라고 가정합니다.
impl Rem<&BigUint> for BigUint {
    type Output = Self;
    fn rem(self, modulus: &BigUint) -> Self::Output {
        let m = modulus.data[0] as u128;
        let base: u128 = 1u128 << 64; // 2^64
        let r = base % m; // 2^64 mod m
        let mut rem: u128 = 0;
        for &limb in self.data.iter().rev() {
            rem = (rem * r + limb as u128) % m;
        }
        BigUint {
            data: vec![rem as u64],
        }
    }
}

impl Mul for BigUint {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        BigUint {
            data: vec![self.data[0] * rhs.data[0]],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shl_single_limb() {
        let a = BigUint { data: vec![3] };
        let shifted = a << 1; // 3 << 1 = 6.
        assert_eq!(shifted.data, vec![6]);
    }

    #[test]
    fn test_shl_whole_limbs() {
        let a = BigUint { data: vec![1] };
        let shifted: BigUint = a << 128; // 1 << 128 should be represented as [0, 0, 1].
        assert_eq!(shifted.data, vec![0, 0, 1]);
    }

    #[test]
    fn test_shl_with_bit_shift() {
        let a = BigUint {
            data: vec![0xFFFF_FFFF_FFFF_FFFF],
        };
        let shifted = a << 4;
        assert_eq!(shifted.data, vec![0xFFFFFFFFFFFFFFF0, 0xF]);
    }
}
