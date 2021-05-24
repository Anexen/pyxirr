use pyxirr;
use rstest::rstest;

mod common;

#[rstest]
fn test_fv_pmt_at_end() {
    let result = pyxirr::pv(0.05 / 12.0, 10.0 * 12.0, -100.0, Some(15692.93), None);
    assert_almost_eq!(result, -100.0006713162);
}

#[rstest]
fn test_fv_pmt_at_begining() {
    let result = pyxirr::pv(0.05 / 12.0, 10.0 * 12.0, -100.0, Some(15692.93), Some(true));
    assert_almost_eq!(result, -60.71677534615);
}

#[rstest]
fn test_fv_zero_rate() {
    let result = pyxirr::pv(0.0, 10.0 * 12.0, -100.0, Some(15692.93), None);
    assert_almost_eq!(result, -3692.93);
}

#[rstest]
fn test_fv_default_pv() {
    let result = pyxirr::pv(0.05 / 12.0, 10.0 * 12.0, -100.0, None, None);
    assert_almost_eq!(result, 9428.1350328234);
}
