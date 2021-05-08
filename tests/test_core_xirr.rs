use rstest::rstest;

use pyxirr;

mod common;
use common::{assert_almost_eq, load_payments};

#[rstest]
#[case("tests/samples/random_100.csv", 29.829404437653)]
#[case("tests/samples/random_1000.csv", 5.508930558032)]
#[case("tests/samples/random_10000.csv", 0.350185149995)]
fn test_samples(#[case] input: &str, #[case] expected: f64) {
    let payments = load_payments(input);
    let result = pyxirr::core::xirr(&payments, None).unwrap();
    assert_almost_eq(result, expected);
}
