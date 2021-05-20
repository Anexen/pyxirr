use rstest::rstest;
use rstest_reuse::{self, *};

use pyo3::Python;
use pyxirr;

mod common;
use common::{xnpv_expected_result, PaymentsLoader};

#[apply(test_samples)]
fn test_xnpv_samples(#[case] input: &str) {
    let rate = 0.1;
    let result = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_records();
        pyxirr::xnpv(rate, payments, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, xnpv_expected_result(rate, input));
}
