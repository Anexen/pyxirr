use rstest::rstest;

use pyxirr;

mod common;
use common::{assert_almost_eq, load_payments};

#[rstest]
#[case(0.1, "tests/samples/random_100.csv", 6488.0382272781)]
#[case(0.5, "tests/samples/random_100.csv", 5686.9590574691)]
#[case(-0.2, "tests/samples/random_100.csv", 7363.0129083468)]
fn test_samples(#[case] rate: f64, #[case] input: &str, #[case] expected: f64) {
    let payments = load_payments(input);
    let result = pyxirr::core::xnpv(rate, &payments).unwrap();
    assert_almost_eq(result, expected);
}
