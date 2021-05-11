#![allow(dead_code)]

use pyxirr::core::Payment;

const MAX_ERROR: f64 = 1e-10;

pub fn assert_almost_eq(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < MAX_ERROR, "{} is not eq to {}", actual, expected)
}

pub fn load_payments(file: &str) -> Vec<Payment> {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(file)
        .unwrap()
        .records()
        .map(|r| r.unwrap())
        .map(|r| Payment { date: r[0].parse().unwrap(), amount: r[1].parse().unwrap() })
        .collect()
}
