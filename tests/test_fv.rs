use pyxirr;
use rstest::rstest;

mod common;

#[rstest]
fn test_fv_pmt_at_end() {
    let result = pyxirr::fv(0.05 / 12., 10. * 12., -100., -100., None);
    assert_almost_eq!(result, 15692.9288943357);
}

#[rstest]
fn test_fv_pmt_at_begining() {
    let result = pyxirr::fv(0.05 / 12., 10. * 12., -100., -100., Some(true));
    assert_almost_eq!(result, 15757.6298441047);
}
