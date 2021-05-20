#![allow(dead_code)]

use rstest_reuse::{self, *};

#[template]
#[rstest]
#[case::unordered("tests/samples/unordered.csv")]
#[case::random_100("tests/samples/random_100.csv")]
#[case::random_1000("tests/samples/random_1000.csv")]
#[case::random_10000("tests/samples/random_10000.csv")]
fn test_samples(#[case] input: &str) {}

pub fn xirr_expected_result(input: &str) -> f64 {
    match input {
        "tests/samples/unordered.csv" => 0.16353715844,
        "tests/samples/random_100.csv" => 29.829404437653,
        "tests/samples/random_1000.csv" => 5.508930558032,
        "tests/samples/random_10000.csv" => 0.350185149995,
        _ => panic!(),
    }
}

pub fn xnpv_expected_result(rate: f64, input: &str) -> f64 {
    // rate is in range [-100, 100] because
    // floating-point types cannot be used in patterns
    let rate = (rate * 100.) as i8;
    match (rate, input) {
        (10, "tests/samples/unordered.csv") => 2218.42566365675,
        (10, "tests/samples/random_100.csv") => 6488.0382272781,
        (10, "tests/samples/random_1000.csv") => 41169.6659983284,
        (10, "tests/samples/random_10000.csv") => -79636.3203295824,
        _ => panic!(),
    }
}

pub fn irr_expected_result(input: &str) -> f64 {
    match input {
        "tests/samples/unordered.csv" => 0.7039842300,
        "tests/samples/random_100.csv" => 2.3320600601,
        "tests/samples/random_1000.csv" => 0.8607558299,
        "tests/samples/random_10000.csv" => 0.3020486969,
        _ => panic!(),
    }
}
