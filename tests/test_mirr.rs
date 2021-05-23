use pyxirr;
use rstest::rstest;

use pyo3::{types::PyList, Python};
mod common;

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
