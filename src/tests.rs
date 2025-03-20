//! 테스트 케이스: Montgomery 모듈러 지수승을 검증합니다.

#[cfg(test)]
mod tests {
    use crate::biguint::BigUint;
    use crate::monty::monty_modpow;

    #[test]
    fn test_monty_modpow_small() {
        // 테스트: 3^13 mod 17, 예상 결과: 12.
        let x = BigUint { data: vec![3] };
        let y = BigUint { data: vec![13] };
        let m = BigUint { data: vec![17] };
        let result = monty_modpow(&x, &y, &m);
        assert_eq!(
            result.data[0] % 17,
            12,
            "3^13 mod 17 should be 12, got {:?}",
            result.data
        );
    }

    #[test]
    fn test_monty_modpow_another() {
        // 테스트: 2^10 mod 19, 예상 결과: 1024 mod 19 = 17.
        let x = BigUint { data: vec![2] };
        let y = BigUint { data: vec![10] };
        let m = BigUint { data: vec![19] };
        let result = monty_modpow(&x, &y, &m);
        assert_eq!(
            result.data[0] % 19,
            17,
            "2^10 mod 19 should be 17, got {:?}",
            result.data
        );
    }

    #[test]
    fn test_monty_modpow_edge() {
        // 엣지 케이스: 1^e mod m는 항상 1이어야 함.
        let x = BigUint::one();
        let y = BigUint { data: vec![12345] };
        let m = BigUint { data: vec![97] };
        let result = monty_modpow(&x, &y, &m);
        assert_eq!(
            result.data[0] % 97,
            1,
            "1^e mod 97 should be 1, got {:?}",
            result.data
        );
    }
}
