use pyo3::prelude::*;
use pyo3::types::*;
use rstest::*;

mod common;
use common::{pd_read_csv, xirr_expected_result, PaymentsLoader};

use pyxirr;

fn get_locals<'p>(py: Python<'p>, input: &str, extra_imports: Option<&[&str]>) -> &'p PyDict {
    let builtins = py.import("builtins").unwrap();
    let data = PaymentsLoader::from_csv(py, input).to_columns();

    let mut locals = vec![
        ("dates", data.0),
        ("amounts", data.1),
        ("iter", builtins.getattr("iter").unwrap()),
        ("list", builtins.getattr("list").unwrap()),
        ("zip", builtins.getattr("zip").unwrap()),
        ("map", builtins.getattr("map").unwrap()),
    ];

    for &name in extra_imports.unwrap_or_default() {
        let module = py.import(name).expect(&format!("{:?} is not installed", name));
        locals.push((name, module))
    }

    locals.into_py_dict(py)
}

#[rstest]
fn test_extract_from_iter() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, None);
        let dates_iter = py.eval("iter(dates)", Some(locals), None).unwrap();
        let amounts_gen = py.eval("(x for x in amounts)", Some(locals), None).unwrap();

        pyxirr::xirr(dates_iter, Some(amounts_gen), None)
    });

    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_tuples() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, None);
        let cash_flow = py.eval("zip(dates, amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(cash_flow, None, None)
    });
    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_lists() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, None);
        let cash_flow = py.eval("map(list, zip(dates, amounts))", Some(locals), None).unwrap();
        pyxirr::xirr(cash_flow, None, None)
    });
    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_dict() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let data = PaymentsLoader::from_csv(py, input).to_dict();
        pyxirr::xirr(data, None, None)
    });
    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_numpy_object_array() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["numpy"]));
        let data = py.eval("numpy.array([dates, amounts])", Some(locals), None).unwrap();
        pyxirr::xirr(data, None, None)
    });

    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_numpy_arrays() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["numpy"]));
        let dates = py.eval("numpy.array(dates)", Some(locals), None).unwrap();
        let amounts = py.eval("numpy.array(amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None)
    });

    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_pandas_dataframe() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let data = pd_read_csv(py, input);
        pyxirr::xirr(data, None, None)
    });

    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_pandas_series() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["pandas"]));
        let dates = py.eval("pandas.Series(dates)", Some(locals), None).unwrap();
        let amounts = py.eval("pandas.Series(amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None)
    });

    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_mixed_iterables() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["pandas", "numpy"]));
        let dates = py.eval("map(pandas.Timestamp, dates)", Some(locals), None).unwrap();
        let amounts = py.eval("numpy.array(amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None)
    });

    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_datetime_and_decimal() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["decimal", "datetime"]));
        let dates = py
            .eval("(datetime.datetime(x.year, x.month, x.day) for x in dates)", Some(locals), None)
            .unwrap();
        let amounts = py.eval("map(decimal.Decimal, amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None)
    });

    assert_almost_eq!(result.unwrap(), xirr_expected_result(input));
}
