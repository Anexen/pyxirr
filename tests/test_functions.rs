use pyxirr;
use rstest::rstest;

mod common;

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
