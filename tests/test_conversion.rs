use pyo3::prelude::*;
use pyo3::{exceptions, types::*};
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
        ("zip", builtins.getattr("zip").unwrap()),
        ("map", builtins.getattr("map").unwrap()),
        ("abs", builtins.getattr("abs").unwrap()),
        ("list", builtins.getattr("list").unwrap()),
        ("int", builtins.getattr("int").unwrap()),
        ("str", builtins.getattr("str").unwrap()),
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
        let locals = get_locals(py, input, Some(&["datetime"]));
        let dates_iter = py
            .eval("(datetime.datetime(x.year, x.month, x.day) for x in dates)", Some(locals), None)
            .unwrap();
        let amounts_gen = py.eval("(x for x in amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates_iter, Some(amounts_gen), None).unwrap().unwrap()
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_tuples() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, None);
        let cash_flow = py.eval("zip(dates, amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(cash_flow, None, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_lists() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, None);
        let cash_flow = py.eval("map(list, zip(dates, amounts))", Some(locals), None).unwrap();
        pyxirr::xirr(cash_flow, None, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_dict() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let data = PaymentsLoader::from_csv(py, input).to_dict();
        pyxirr::xirr(data, None, None).unwrap().unwrap()
    });
    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_numpy_object_array() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["numpy"]));
        let data = py.eval("numpy.array([dates, amounts])", Some(locals), None).unwrap();
        pyxirr::xirr(data, None, None).unwrap().unwrap()
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_numpy_arrays() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["numpy"]));
        let dates = py.eval("numpy.array(dates)", Some(locals), None).unwrap();
        let amounts = py.eval("numpy.array(amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None).unwrap().unwrap()
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_pandas_dataframe() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let data = pd_read_csv(py, input);
        pyxirr::xirr(data, None, None).unwrap().unwrap()
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_pandas_series() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["pandas"]));
        let dates = py.eval("pandas.Series(dates)", Some(locals), None).unwrap();
        let amounts = py.eval("pandas.Series(amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None).unwrap().unwrap()
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_pandas_series_with_datetime_index() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["pandas"]));
        let dates = py
            .eval("pandas.Series(amounts, index=pandas.to_datetime(dates))", Some(locals), None)
            .unwrap();
        pyxirr::xirr(dates, None, None).unwrap().unwrap()
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_failed_extract_from_pandas_series_with_int64_index() {
    let input = "tests/samples/unordered.csv";
    Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["pandas"]));
        let dates = py.eval("pandas.Series(amounts)", Some(locals), None).unwrap();
        let err = pyxirr::xirr(dates, None, None).unwrap_err();
        assert!(err.is_instance::<exceptions::PyTypeError>(py));
    });
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_mixed_iterables() {
    let input = "tests/samples/unordered.csv";
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, input, Some(&["pandas", "numpy"]));
        let dates = py.eval("map(pandas.Timestamp, dates)", Some(locals), None).unwrap();
        let amounts = py.eval("numpy.array(amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None).unwrap().unwrap()
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
fn test_extract_from_non_float() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let input = "tests/samples/unordered.csv";
    let expected = xirr_expected_result(input);

    let locals = get_locals(py, input, Some(&["decimal"]));
    let dates = locals.get_item("dates").unwrap();

    let amounts = py.eval("map(decimal.Decimal, amounts)", Some(locals), None).unwrap();
    let result = pyxirr::xirr(dates, Some(amounts), None).unwrap().unwrap();
    assert_almost_eq!(result, expected);

    let amounts = py.eval("map(int, amounts)", Some(locals), None).unwrap();
    let result = pyxirr::xirr(dates, Some(amounts), None).unwrap().unwrap();
    assert_almost_eq!(result, expected);

    let amounts = py.eval("map(str, amounts)", Some(locals), None).unwrap();
    let err = pyxirr::xirr(dates, Some(amounts), None).unwrap_err();
    assert!(err.is_instance::<exceptions::PyTypeError>(py));
}

#[rstest]
fn test_payments_different_sign() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let input = "tests/samples/unordered.csv";
    let locals = get_locals(py, input, None);
    let dates = locals.get_item("dates").unwrap();

    let amounts = py.eval("(abs(x) for x in amounts)", Some(locals), None).unwrap();
    let err = pyxirr::xirr(dates, Some(amounts), None).unwrap_err();
    assert!(err.is_instance::<pyxirr::InvalidPaymentsError>(py));

    let amounts = py.eval("(-abs(x) for x in amounts)", Some(locals), None).unwrap();
    let err = pyxirr::xirr(dates, Some(amounts), None).unwrap_err();
    assert!(err.is_instance::<pyxirr::InvalidPaymentsError>(py));
}

#[rstest]
fn test_arrays_of_dirrerent_lengths() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let input = "tests/samples/unordered.csv";
    let locals = get_locals(py, input, None);
    let dates = locals.get_item("dates").unwrap();
    let amounts = py.eval("amounts[:-2]", Some(locals), None).unwrap();
    let err = pyxirr::xirr(dates, Some(amounts), None).unwrap_err();
    assert!(err.is_instance::<pyxirr::InvalidPaymentsError>(py));
}
