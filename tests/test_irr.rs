use rstest::rstest;
use rstest_reuse::{self, *};

use pyo3::{types::PyList, Python};
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

// test cases from numpy_finance.irr
#[rstest]
#[case(&[-100, 39, 59, 55, 20], 0.28094842116)]
#[case(&[-100, 0, 0, 74], -0.09549583034)]
#[case(&[-100, 100, 0, -7], -0.08329966618)]
fn test_irr_works(#[case] input: &[i64], #[case] expected: f64) {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let values = PyList::new(py, input);
    let result = pyxirr::irr(values, None).unwrap().unwrap();
    assert_almost_eq!(result, expected);
}
