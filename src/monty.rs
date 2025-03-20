//! Montgomery 모듈러 산술 및 지수승.
//!
//! 오른쪽부터 4비트 윈도우 방식을 사용한 Montgomery 곱셈 기반 모듈러 지수승 구현입니다.
//! 내부 곱셈 루틴은 "parallel" feature가 활성화된 경우, 여러 limb 청크로 분할하여 병렬 처리하며,
//! 청크 간의 carry 처리는 순차적으로 수행됩니다.

use crate::big_digit::{BigDigit, DoubleBigDigit, BITS};
use crate::biguint::BigUint;
use core::ops::Shl; // Shl 트레이트 임포트

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Montgomery Reduction에 필요한 매개변수를 저장하는 구조체.
/// k0 = -m⁻¹ mod 2^BITS 값을 미리 계산합니다.
pub struct MontyReducer {
    pub n0inv: BigDigit,
}

/// 홀수 m에 대해 k0 = -m⁻¹ mod 2^BITS를 계산합니다.
fn inv_mod_alt(b: BigDigit) -> BigDigit {
    assert_ne!(b & 1, 0, "b must be odd");
    let mut k0 = BigDigit::wrapping_sub(2, b);
    let mut t = b - 1;
    let mut i = 1;
    while i < BITS {
        t = t.wrapping_mul(t);
        k0 = k0.wrapping_mul(t.wrapping_add(1));
        i <<= 1;
    }
    k0.wrapping_neg()
}

impl MontyReducer {
    pub fn new(n: &BigUint) -> Self {
        let n0inv = inv_mod_alt(n.data[0]);
        MontyReducer { n0inv }
    }
}

/// Sequential inner multiplication: 두 limb 벡터 x와 y의 곱을 z에 더하며, 최종 carry를 반환합니다.
fn add_mul_vvw_seq(z: &mut [BigDigit], x: &[BigDigit], y: BigDigit) -> BigDigit {
    let mut c = 0;
    for (zi, xi) in z.iter_mut().zip(x.iter()) {
        let (z1, z0) = mul_add_www(*xi, y, *zi);
        let (c_new, zi_new) = add_ww(z0, c, 0);
        *zi = zi_new;
        c = c_new + z1;
    }
    c
}

/// Parallel inner multiplication: x와 y의 곱을 limb 청크 단위로 분할하여 병렬 처리합니다.
/// 각 청크의 carry는 나중에 순차적으로 결합합니다.
#[cfg(feature = "parallel")]
fn add_mul_vvw(z: &mut [BigDigit], x: &[BigDigit], y: BigDigit) -> BigDigit {
    use rayon::prelude::*;
    let len = x.len();
    let chunk_size = 64.min(len); // 청크 크기: 64 limb (필요에 따라 조정)
    let num_chunks = (len + chunk_size - 1) / chunk_size;
    let mut carries = vec![0u64; num_chunks];

    z.par_chunks_mut(chunk_size)
        .enumerate()
        .for_each(|(chunk_idx, chunk)| {
            let start = chunk_idx * chunk_size;
            let mut c: BigDigit = 0;
            for j in 0..chunk.len() {
                let global_index = start + j;
                let (z1, z0) = mul_add_www(x[global_index], y, chunk[j]);
                let (c_new, zi_new) = add_ww(z0, c, 0);
                chunk[j] = zi_new;
                c = c_new + z1;
            }
            carries[chunk_idx] = c;
        });
    let mut carry = 0;
    for &c in &carries {
        carry = carry.wrapping_add(c);
    }
    carry
}

#[cfg(not(feature = "parallel"))]
fn add_mul_vvw(z: &mut [BigDigit], x: &[BigDigit], y: BigDigit) -> BigDigit {
    add_mul_vvw_seq(z, x, y)
}

/// 두 limb와 carry를 더하는 함수, (carry, sum)을 반환합니다.
#[inline(always)]
fn add_ww(x: BigDigit, y: BigDigit, c: BigDigit) -> (BigDigit, BigDigit) {
    let yc = y.wrapping_add(c);
    let sum = x.wrapping_add(yc);
    let carry = if sum < x || yc < y { 1 } else { 0 };
    (carry, sum)
}

/// x와 y를 곱하고 c를 더한 결과의 high, low 부분을 반환합니다.
#[inline(always)]
fn mul_add_www(x: BigDigit, y: BigDigit, c: BigDigit) -> (BigDigit, BigDigit) {
    let prod = x as DoubleBigDigit * y as DoubleBigDigit + c as DoubleBigDigit;
    (((prod >> BITS) as BigDigit), prod as BigDigit)
}

/// Sequential subtraction over slices with carry propagation.
#[inline(always)]
fn sub_vv(z: &mut [BigDigit], x: &[BigDigit], y: &[BigDigit]) -> BigDigit {
    let mut c = 0;
    for (i, (xi, yi)) in x.iter().zip(y.iter()).enumerate().take(z.len()) {
        let zi = xi.wrapping_sub(*yi).wrapping_sub(c);
        z[i] = zi;
        c = ((yi & !xi) | ((yi | !xi) & zi)) >> (BITS - 1);
    }
    c
}

/// Montgomery multiplication.
/// z = x * y * R⁻¹ mod m, where R = 2^(BITS * n) (n = m.data.len()).
pub fn montgomery(x: &BigUint, y: &BigUint, m: &BigUint, k: BigDigit, n: usize) -> BigUint {
    // x, y, m는 모두 n limb를 가져야 합니다.
    assert!(
        x.data.len() == n && y.data.len() == n && m.data.len() == n,
        "Length mismatch"
    );
    let mut z = BigUint {
        data: vec![0; n * 2],
    };
    let mut c: BigDigit = 0;
    for i in 0..n {
        let c2 = add_mul_vvw(&mut z.data[i..n + i], &x.data, y.data[i]);
        let t = z.data[i].wrapping_mul(k);
        let c3 = add_mul_vvw(&mut z.data[i..n + i], &m.data, t);
        let cx = c.wrapping_add(c2);
        let cy = cx.wrapping_add(c3);
        z.data[n + i] = cy;
        c = if cx < c2 || cy < c3 { 1 } else { 0 };
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

/// Montgomery 모듈러 지수승.
/// 오른쪽부터 4비트 윈도우 방식을 사용하여, Montgomery 곱셈을 반복한 후
/// 최종 결과를 일반 숫자로 변환합니다.
/// 지수 y의 실제 유효 비트 수만 처리하도록 합니다.
/// Montgomery 모듈러 지수승 using 오른쪽부터 4비트 윈도우 방식을 사용합니다.
/// y의 유효 비트 수만 처리하도록 수정하며, y의 마지막 limb를 왼쪽 패딩하여 64비트로 맞춥니다.
pub fn monty_modpow(x: &BigUint, y: &BigUint, m: &BigUint) -> BigUint {
    assert!(m.data[0] & 1 == 1, "Modulus must be odd");
    let mr = MontyReducer::new(m);
    let num_words = m.data.len();

    // x를 num_words 크기로 맞춤.
    let mut x = x.clone();
    if x.data.len() > num_words {
        x.data.truncate(num_words);
    }
    if x.data.len() < num_words {
        x.data.resize(num_words, 0);
    }

    // rr = 2^(2 * num_words * BITS) mod m.
    let mut rr = BigUint::one();
    rr = rr.shl(2 * num_words as u64 * BITS as u64) % m;
    if rr.data.len() < num_words {
        rr.data.resize(num_words, 0);
    }
    let mut one = BigUint::one();
    one.data.resize(num_words, 0);

    #[cfg(debug_assertions)]
    {
        eprintln!("DEBUG: rr = {:?}", rr.data);
        eprintln!("DEBUG: one = {:?}", one.data);
    }

    let window_bits = 4;
    let table_size = 1 << window_bits;

    // Precompute table: powers[i] = x^i in Montgomery form.
    let mut powers = Vec::with_capacity(table_size);
    powers.push(montgomery(&one, &rr, m, mr.n0inv, num_words));
    powers.push(montgomery(&x, &rr, m, mr.n0inv, num_words));
    for i in 2..table_size {
        let r = montgomery(&powers[i - 1], &powers[1], m, mr.n0inv, num_words);
        powers.push(r);
    }

    #[cfg(debug_assertions)]
    {
        for (i, p) in powers.iter().enumerate() {
            eprintln!("DEBUG: powers[{}] = {:?}", i, p.data);
        }
    }

    // y의 유효 비트 수.
    let total_bits = y.bit_length();
    #[cfg(debug_assertions)]
    eprintln!("DEBUG: total_bits = {}", total_bits);

    // y가 한 limb인 경우, y.data.last()의 값에 (64 - total_bits) 만큼 왼쪽 패딩.
    let mut exp = if let Some(&last) = y.data.last() {
        last << (BITS - total_bits)
    } else {
        0
    };
    #[cfg(debug_assertions)]
    eprintln!("DEBUG: initial exp (padded) = {}", exp);
    let mut processed = 0;

    // z를 Montgomery form의 1로 초기화.
    let mut z = powers[0].clone();
    z.data.resize(num_words, 0);

    while processed < total_bits {
        if processed > 0 {
            // Montgomery squaring: 윈도우 크기만큼 자리 올리기.
            let mut tmp = montgomery(&z, &z, m, mr.n0inv, num_words);
            tmp = montgomery(&tmp, &tmp, m, mr.n0inv, num_words);
            tmp = montgomery(&tmp, &tmp, m, mr.n0inv, num_words);
            z = montgomery(&tmp, &tmp, m, mr.n0inv, num_words);
        }
        let idx = (exp >> (BITS - window_bits)) as usize;
        #[cfg(debug_assertions)]
        eprintln!(
            "DEBUG: processed = {}, exp = {}, idx = {}",
            processed, exp, idx
        );
        z = montgomery(&z, &powers[idx], m, mr.n0inv, num_words);
        exp <<= window_bits;
        processed += window_bits;
    }

    let mut zz = montgomery(&z, &one, m, mr.n0inv, num_words);
    zz.normalize();
    if zz >= *m {
        zz.data[0] %= m.data[0];
    }
    zz.normalize();
    #[cfg(debug_assertions)]
    eprintln!("DEBUG: Final result = {:?}", zz.data);
    zz
}
