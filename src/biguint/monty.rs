use alloc::vec::Vec;
use core::mem;
use core::ops::Shl;
use num_traits::One;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

use crate::big_digit::{self, BigDigit, DoubleBigDigit};
use crate::biguint::BigUint;

struct MontyReducer {
    n0inv: BigDigit,
}

fn inv_mod_alt(b: BigDigit) -> BigDigit {
    assert_ne!(b & 1, 0);

    let mut k0 = BigDigit::wrapping_sub(2, b);
    let mut t = b - 1;
    let mut i = 1;
    while i < big_digit::BITS {
        t = t.wrapping_mul(t);
        k0 = k0.wrapping_mul(t + 1);
        i <<= 1;
    }
    debug_assert_eq!(k0.wrapping_mul(b), 1);
    k0.wrapping_neg()
}

impl MontyReducer {
    fn new(n: &BigUint) -> Self {
        let n0inv = inv_mod_alt(n.data[0]);
        MontyReducer { n0inv }
    }
}

/// Computes z mod m = x * y * 2 ** (-n*_W) mod m
/// assuming k = -1/m mod 2**_W
/// See Gueron, "Efficient Software Implementations of Modular Exponentiation".
/// <https://eprint.iacr.org/2011/239.pdf>
/// In the terminology of that paper, this is an "Almost Montgomery Multiplication":
/// x and y are required to satisfy 0 <= z < 2**(n*_W) and then the result
/// z is guaranteed to satisfy 0 <= z < 2**(n*_W), but it may not be < m.
/// montgomery 함수(기존 그대로)
#[allow(clippy::many_single_char_names)]
fn montgomery(x: &BigUint, y: &BigUint, m: &BigUint, k: BigDigit, n: usize) -> BigUint {
    assert!(
        x.data.len() == n && y.data.len() == n && m.data.len() == n,
        "{:?} {:?} {:?} {}",
        x,
        y,
        m,
        n
    );

    let mut z = BigUint::ZERO;
    z.data.resize(n * 2, 0);

    let mut c: BigDigit = 0;
    for i in 0..n {
        // 여기서 add_mul_vvw가 호출됨. (병렬화 대상)
        let c2 = add_mul_vvw(&mut z.data[i..n + i], &x.data, y.data[i]);
        let t = z.data[i].wrapping_mul(k);
        let c3 = add_mul_vvw(&mut z.data[i..n + i], &m.data, t);
        let cx = c.wrapping_add(c2);
        let cy = cx.wrapping_add(c3);
        z.data[n + i] = cy;
        if cx < c2 || cy < c3 {
            c = 1;
        } else {
            c = 0;
        }
    }

    if c == 0 {
        z.data = z.data[n..].to_vec();
    } else {
        {
            let (first, second) = z.data.split_at_mut(n);
            sub_vv(first, second, &m.data);
        }
        z.data = z.data[..n].to_vec();
    }

    z
}

#[inline(always)]
fn add_mul_vvw(z: &mut [BigDigit], x: &[BigDigit], y: BigDigit) -> BigDigit {
    #[cfg(feature = "parallel")]
    {
        add_mul_vvw_parallel(z, x, y)
    }
    #[cfg(not(feature = "parallel"))]
    {
        add_mul_vvw_serial(z, x, y)
    }
}

#[inline(always)]
fn add_mul_vvw_serial(z: &mut [BigDigit], x: &[BigDigit], y: BigDigit) -> BigDigit {
    let mut c = 0;
    for (zi, xi) in z.iter_mut().zip(x.iter()) {
        let (z1, z0) = mul_add_www(*xi, y, *zi);
        let (c_, zi_) = add_ww(z0, c, 0);
        *zi = zi_;
        c = c_ + z1;
    }
    c
}

#[cfg(feature = "parallel")]
fn add_mul_vvw_parallel(z: &mut [BigDigit], x: &[BigDigit], y: BigDigit) -> BigDigit {
    use core::sync::atomic::{AtomicU64, Ordering};

    let n = x.len();
    assert!(z.len() >= n);

    // 1) 먼저 (x_i * y) + z_i를 병렬로 계산하여 128비트 값(partial_sum)을 구함
    //    partial_sum[i] = (carry_hi << BITS) + lo
    //    여기서 carry_hi 는 mul_add_www에서 나오는 상위 BigDigit
    //    lo 는 최종 z[i]에 들어갈 값 (add_ww 과정 일부 포함)
    let partials: Vec<u128> = z
        .par_iter()
        .zip(x.par_iter())
        .map(|(&zi, &xi)| {
            // 기존 mul_add_www + add_ww 과정을 한꺼번에 수행
            let (mul_hi, mul_lo) = mul_add_www(xi, y, zi);
            // 이 시점에서 mul_lo + carry_c (0)은 아래 add_ww와 동일하므로
            // z1, z0 = add_ww(mul_lo, 0, 0) 형태
            // 결국 z1은 mul_lo 오버플로 체크 정도지만, 여기서는 0이니 단순화
            ((mul_hi as u128) << big_digit::BITS) | (mul_lo as u128)
        })
        .collect();

    // 2) 위 단계에서 carry를 제대로 전파하지 않았으므로,
    //    이제 직렬로 partials를 순회하며 carry를 전파한다.
    //    partials[i]는 최대 2^BITS + 2^BITS = 2^(BITS+1) 정도이므로
    //    u128에 충분히 들어갑니다.(BITS가 64라면 128비트 필요)
    let mut c = 0u128;
    for i in 0..n {
        let sum = partials[i] + c;
        let z_new = sum as BigDigit; // 하위 BITS
        let carry_out = sum >> big_digit::BITS; // 상위 BITS
        z[i] = z_new;
        c = carry_out;
    }

    // 마지막 carry 반환 (0 or 1 정도일 것)
    c as BigDigit
}

/// sub_vv (직렬)
#[inline(always)]
fn sub_vv(z: &mut [BigDigit], x: &[BigDigit], y: &[BigDigit]) -> BigDigit {
    let mut c = 0;
    for (i, (xi, yi)) in x.iter().zip(y.iter()).enumerate().take(z.len()) {
        let zi = xi.wrapping_sub(*yi).wrapping_sub(c);
        z[i] = zi;
        // Hacker's Delight 의 오버플로 검출 trick
        c = ((yi & !xi) | ((yi | !xi) & zi)) >> (big_digit::BITS - 1);
    }
    c
}
/// add_ww (직렬)
#[inline(always)]
fn add_ww(x: BigDigit, y: BigDigit, c: BigDigit) -> (BigDigit, BigDigit) {
    let yc = y.wrapping_add(c);
    let z0 = x.wrapping_add(yc);
    let z1 = if z0 < x || yc < y { 1 } else { 0 };
    (z1, z0)
}

/// mul_add_www (직렬)
#[inline(always)]
fn mul_add_www(x: BigDigit, y: BigDigit, c: BigDigit) -> (BigDigit, BigDigit) {
    let z = x as DoubleBigDigit * y as DoubleBigDigit + c as DoubleBigDigit;
    ((z >> big_digit::BITS) as BigDigit, z as BigDigit)
}

/// Calculates x ** y mod m using a fixed, 4-bit window.
#[allow(clippy::many_single_char_names)]
pub(super) fn monty_modpow(x: &BigUint, y: &BigUint, m: &BigUint) -> BigUint {
    assert!(m.data[0] & 1 == 1);
    let mr = MontyReducer::new(m);
    let num_words = m.data.len();

    let mut x = x.clone();
    if x.data.len() > num_words {
        x %= m;
    }
    if x.data.len() < num_words {
        x.data.resize(num_words, 0);
    }

    // rr = 2^(2*W*len(m)) mod m
    let mut rr = BigUint::one();
    rr = (rr.shl(2 * num_words as u64 * u64::from(big_digit::BITS))) % m;
    if rr.data.len() < num_words {
        rr.data.resize(num_words, 0);
    }
    // one = 1, with equal length to that of m
    let mut one = BigUint::one();
    one.data.resize(num_words, 0);

    let n = 4;
    // powers[i] = x^i
    let mut powers = Vec::with_capacity(1 << n);
    powers.push(montgomery(&one, &rr, m, mr.n0inv, num_words));
    powers.push(montgomery(&x, &rr, m, mr.n0inv, num_words));
    for i in 2..1 << n {
        let r = montgomery(&powers[i - 1], &powers[1], m, mr.n0inv, num_words);
        powers.push(r);
    }

    // initialize z = 1 (Montgomery 1)
    let mut z = powers[0].clone();
    z.data.resize(num_words, 0);
    let mut zz = BigUint::ZERO;
    zz.data.resize(num_words, 0);

    // windowed exponentiation
    for i in (0..y.data.len()).rev() {
        let mut yi = y.data[i];
        let mut j = 0;
        while j < big_digit::BITS {
            if i != y.data.len() - 1 || j != 0 {
                zz = montgomery(&z, &z, m, mr.n0inv, num_words);
                z = montgomery(&zz, &zz, m, mr.n0inv, num_words);
                zz = montgomery(&z, &z, m, mr.n0inv, num_words);
                z = montgomery(&zz, &zz, m, mr.n0inv, num_words);
            }
            zz = montgomery(
                &z,
                &powers[(yi >> (big_digit::BITS - n)) as usize],
                m,
                mr.n0inv,
                num_words,
            );
            mem::swap(&mut z, &mut zz);
            yi <<= n;
            j += n;
        }
    }

    // convert to regular number
    zz = montgomery(&z, &one, m, mr.n0inv, num_words);
    zz.normalize();

    // 마지막으로 한 번 더 m보다 큰지 확인하고 빼주기 (Go 이슈 #13907 대응)
    if zz >= *m {
        zz -= m;
        if zz >= *m {
            zz %= m;
        }
    }
    zz.normalize();
    zz
}
