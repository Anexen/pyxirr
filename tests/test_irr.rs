use rstest::rstest;
use rstest_reuse::{self, *};

use pyo3::Python;
use pyxirr;

mod common;
use common::{irr_expected_result, PaymentsLoader};

#[apply(test_samples)]
fn test_irr_samples(#[case] input: &str) {
    let result = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_columns();
        pyxirr::irr(payments.1, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, irr_expected_result(input));
}
