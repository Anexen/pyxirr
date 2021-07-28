use rstest::rstest;
use rstest_reuse::{self, *};

use pyo3::types::{IntoPyDict, PyDate};
use pyo3::Python;

use pyxirr;

mod common;
use common::{xirr_expected_result, xnpv_expected_result, PaymentsLoader};

#[apply(test_samples)]
fn test_xnpv_samples(#[case] input: &str) {
    let rate = 0.1;
    let result = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_records();
        pyxirr::xnpv(rate, payments, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, xnpv_expected_result(rate, input));
}

#[apply(test_samples)]
fn test_xirr_samples(#[case] input: &str) {
    let result = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_records();
        pyxirr::xirr(payments, None, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
fn test_xfv() {
    // http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-XFV-function
    Python::with_gil(|py| {
        let result = pyxirr::xfv(
            PyDate::new(py, 2011, 2, 1).unwrap().into(),
            PyDate::new(py, 2011, 3, 1).unwrap().into(),
            PyDate::new(py, 2012, 2, 1).unwrap().into(),
            0.00142,
            0.00246,
            100000.,
        );
        assert_almost_eq!(result.unwrap(), 100235.088391894);
    });
}

#[rstest]
fn test_xnfv() {
    Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, "tests/samples/xnfv.csv").to_records();
        let result = pyxirr::xnfv(0.0250, payments, None).unwrap();
        assert_almost_eq!(result, 57238.1249299303);
    });
}

#[rstest]
fn test_sum_xfv_eq_xnfv() {
    Python::with_gil(|py| {
        let rate = 0.0250;
        let (dates, amounts) = PaymentsLoader::from_csv(py, "tests/samples/xnfv.csv").to_columns();

        let xnfv_result = pyxirr::xnfv(rate, dates, Some(amounts)).unwrap();

        let builtins = py.import("builtins").unwrap();
        let locals = vec![
            ("dates", dates),
            ("min", builtins.getattr("min").unwrap()),
            ("max", builtins.getattr("max").unwrap()),
        ]
        .into_py_dict(py);

        let min_date = py.eval("min(dates)", Some(locals), None).unwrap();
        let max_date = py.eval("max(dates)", Some(locals), None).unwrap();
        let sum_xfv_result: f64 = dates
            .iter()
            .unwrap()
            .zip(amounts.iter().unwrap())
            .map(|(d, a)| {
                pyxirr::xfv(
                    min_date.extract().unwrap(),
                    d.unwrap().extract().unwrap(),
                    max_date.extract().unwrap(),
                    rate,
                    rate,
                    a.unwrap().extract().unwrap(),
                )
                .unwrap()
            })
            .sum();

        assert_almost_eq!(xnfv_result, sum_xfv_result);
    });
}
