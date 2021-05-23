use pyxirr;
use rstest::rstest;

mod common;

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
