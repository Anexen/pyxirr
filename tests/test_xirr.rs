use rstest::rstest;
use rstest_reuse::{self, *};

use pyo3::Python;
use pyxirr;

mod common;
use common::{xirr_expected_result, PaymentsLoader};

#[apply(test_samples)]
fn test_xirr_samples(#[case] input: &str) {
    let result = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_records();
        pyxirr::xirr(payments, None, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, xirr_expected_result(input));
}
