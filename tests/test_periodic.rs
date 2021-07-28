use rstest::rstest;
use rstest_reuse::{self, *};

use pyo3::{types::PyList, Python};

use pyxirr;

mod common;
use common::{irr_expected_result, PaymentsLoader};

const INTEREST_RATE: f64 = 0.05;
const PERIODS: f64 = 10.0;
const PAYMENT: f64 = -50_000.0;
const PV: f64 = 100_000.0;
const FV: f64 = 110_000.0;

#[rstest]
fn test_fv_macro_working() {
    assert_future_value!(INTEREST_RATE, PERIODS, -12950.4574965456, PV, None, None);
    assert_future_value!(INTEREST_RATE, PERIODS, -12333.7690443292, PV, None, Some(true));
    assert_future_value!(INTEREST_RATE, PERIODS, -21695.9607427458, PV, Some(FV), None);
    assert_future_value!(INTEREST_RATE, PERIODS, -20662.8197549960, PV, Some(FV), Some(true));
}

// ------------ FV ----------------

#[rstest]
fn test_fv_pmt_at_end() {
    let result = pyxirr::fv(0.05 / 12.0, 10.0 * 12.0, -100.0, -100.0, None);
    assert_almost_eq!(result, 15692.9288943357);
}

#[rstest]
fn test_fv_pmt_at_begining() {
    let result = pyxirr::fv(0.05 / 12.0, 10.0 * 12.0, -100.0, -100.0, Some(true));
    assert_almost_eq!(result, 15757.6298441047);
}

#[rstest]
fn test_fv_zero_rate() {
    let result = pyxirr::fv(0.0, 10.0 * 12.0, -100.0, -100.0, None);
    assert_almost_eq!(result, 12100.0);
}

// ------------ PV ----------------

#[rstest]
fn test_pv_pmt_at_end() {
    let result = pyxirr::pv(0.05 / 12.0, 10.0 * 12.0, -100.0, Some(15692.93), None);
    assert_almost_eq!(result, -100.0006713162);
}

#[rstest]
fn test_pv_pmt_at_begining() {
    let result = pyxirr::pv(0.05 / 12.0, 10.0 * 12.0, -100.0, Some(15692.93), Some(true));
    assert_almost_eq!(result, -60.71677534615);
}

#[rstest]
fn test_pv_zero_rate() {
    let result = pyxirr::pv(0.0, 10.0 * 12.0, -100.0, Some(15692.93), None);
    assert_almost_eq!(result, -3692.93);
}

#[rstest]
fn test_pv_default_pv() {
    let result = pyxirr::pv(0.05 / 12.0, 10.0 * 12.0, -100.0, None, None);
    assert_almost_eq!(result, 9428.1350328234);
}

// ------------ NPV ----------------

#[rstest]
fn test_npv_works() {
    Python::with_gil(|py| {
        let values = PyList::new(py, &[-40_000., 5_000., 8_000., 12_000., 30_000.]);
        let result = pyxirr::npv(0.08, values, None).unwrap().unwrap();
        assert_almost_eq!(result, 3065.222668179);
    });
}

#[rstest]
fn test_npv_start_from_zero() {
    Python::with_gil(|py| {
        let values = PyList::new(py, &[-40_000., 5_000., 8_000., 12_000., 30_000.]);
        let result = pyxirr::npv(0.08, values, Some(false)).unwrap().unwrap();
        assert_almost_eq!(result, 2838.169137203);
    });
}

#[rstest]
fn test_npv_zero_rate() {
    Python::with_gil(|py| {
        let values = &[-40_000., 5_000., 8_000., 12_000., 30_000.];
        let result = pyxirr::npv(0., PyList::new(py, values), Some(false));
        assert_almost_eq!(result.unwrap().unwrap(), 15_000.0);
    });
}

// ------------ PMT ----------------

#[rstest]
fn test_pmt_pmt_at_end() {
    let pmt = pyxirr::pmt(INTEREST_RATE, PERIODS, PV, None, None);
    assert_future_value!(INTEREST_RATE, PERIODS, pmt, PV, None, None);
}

#[rstest]
fn test_pmt_pmt_at_begining() {
    let pmt = pyxirr::pmt(INTEREST_RATE, PERIODS, PV, None, Some(true));
    assert_future_value!(INTEREST_RATE, PERIODS, pmt, PV, None, Some(true));
}

#[rstest]
fn test_pmt_non_zero_fv() {
    let pmt = pyxirr::pmt(INTEREST_RATE, PERIODS, PV, Some(FV), None);
    assert_future_value!(INTEREST_RATE, PERIODS, pmt, PV, Some(FV), None);
}

#[rstest]
fn test_pmt_zero_rate() {
    let pmt = pyxirr::pmt(0.0, PERIODS, PV, Some(FV), None);
    assert_future_value!(0.0, PERIODS, pmt, PV, Some(FV), None);
}

// ------------ NPER ----------------

#[rstest]
fn test_nper_pmt_at_end() {
    let nper = pyxirr::nper(INTEREST_RATE, PAYMENT, PV, None, None);
    assert_future_value!(INTEREST_RATE, nper, PAYMENT, PV, None, None);
}

#[rstest]
fn test_nper_pmt_at_begining() {
    let nper = pyxirr::nper(INTEREST_RATE, PAYMENT, PV, None, Some(true));
    assert_future_value!(INTEREST_RATE, nper, PAYMENT, PV, None, Some(true));
}

#[rstest]
fn test_nper_non_zero_fv() {
    let nper = pyxirr::nper(INTEREST_RATE, PAYMENT, PV, Some(FV), None);
    assert_future_value!(INTEREST_RATE, nper, PAYMENT, PV, Some(FV), None);
}

#[rstest]
fn test_nper_zero_rate() {
    let nper = pyxirr::nper(0.0, PAYMENT, PV, Some(FV), None);
    assert_future_value!(0.0, nper, PAYMENT, PV, Some(FV), None);
}

// ------------ NFV ----------------

#[rstest]
fn test_nfv() {
    // example from https://www.youtube.com/watch?v=775ljhriB8U
    Python::with_gil(|py| {
        let amounts = PyList::new(py, &[1050.0, 1350.0, 1350.0, 1450.0]);
        let result = pyxirr::nfv(0.03, 6.0, amounts);
        assert_almost_eq!(result.unwrap(), 5750.16, 0.01);
    });
}

// ------------ IRR ----------------

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

#[apply(test_samples)]
fn test_irr_samples(#[case] input: &str) {
    let result = Python::with_gil(|py| {
        let payments = PaymentsLoader::from_csv(py, input).to_columns();
        pyxirr::irr(payments.1, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, irr_expected_result(input));
}

// ------------ MIRR ----------------

#[rstest]
fn test_mirr_works() {
    let result = Python::with_gil(|py| {
        let values = PyList::new(py, &[-1000, 100, 250, 500, 500]);
        pyxirr::mirr(values, 0.1, 0.1).unwrap().unwrap()
    });
    assert_almost_eq!(result, 0.10401626745);
}

#[rstest]
fn test_mirr_same_sign() {
    Python::with_gil(|py| {
        let values = PyList::new(py, &[100_000.0, 50_000.0, 25_000.0]);
        assert!(pyxirr::mirr(values, 0.1, 0.1).unwrap().is_none());
        let values = PyList::new(py, &[-100_000.0, -50_000.0, -25_000.0]);
        assert!(pyxirr::mirr(values, 0.1, 0.1).unwrap().is_none());
    });
}
