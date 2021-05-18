use rstest::rstest;

use pyo3::Python;
use pyxirr;

mod common;

#[rstest]
#[case(0.1, "tests/samples/random_100.csv", 6488.0382272781)]
#[case(0.5, "tests/samples/random_100.csv", 5686.9590574691)]
#[case(-0.2, "tests/samples/random_100.csv", 7363.0129083468)]
fn test_samples(#[case] rate: f64, #[case] input: &str, #[case] expected: f64) {
    let result = Python::with_gil(|py| {
        let loader = common::PaymentsLoader::from_csv(py, input);
        let payments = loader.to_columns();
        pyxirr::xnpv(rate, payments.0, Some(payments.1)).unwrap()
    });
    assert_almost_eq!(result, expected);
}
