use pyo3::prelude::*;
use pyo3::{exceptions, types::*};
use rstest::*;

mod common;
use common::{pd_read_csv, xirr_expected_result, PaymentsLoader};

use pyxirr;

type Payments = (Py<PyAny>, Py<PyAny>);
const INPUT: &str = "tests/samples/unordered.csv";

#[fixture]
fn payments(#[default(INPUT)] input: &str) -> Payments {
    Python::with_gil(|py| {
        let (dates, amounts) = PaymentsLoader::from_csv(py, input).to_columns();
        (dates.into(), amounts.into())
    })
}

fn get_locals<'p>(py: Python<'p>, extra_imports: Option<&[&str]>) -> &'p PyDict {
    let builtins = py.import("builtins").unwrap();
    let data = payments(INPUT);
    let locals = py_dict!(py, "dates" => data.0, "amounts" => data.1);
    let locals = py_dict_merge!(py, locals, builtins.dict());

    for &name in extra_imports.unwrap_or_default() {
        let module = py.import(name).expect(&format!("{:?} is not installed", name));
        locals.set_item(name, module).unwrap();
    }

    locals
}

#[rstest]
fn test_extract_from_iter() {
    let result: f64 = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["datetime"]));
        let dates_iter = py
            .eval("(datetime.datetime(x.year, x.month, x.day) for x in dates)", Some(locals), None)
            .unwrap();
        let amounts_gen = py.eval("(x for x in amounts)", Some(locals), None).unwrap();
        pyxirr_call!(py, "xirr", (dates_iter, amounts_gen))
    });

    assert_almost_eq!(result, xirr_expected_result(INPUT));
}

#[rstest]
fn test_extract_from_tuples(payments: Payments) {
    let result: f64 = Python::with_gil(|py| pyxirr_call!(py, "xirr", payments));
    assert_almost_eq!(result, xirr_expected_result(INPUT));
}

#[rstest]
fn test_extract_from_lists() {
    let result: f64 = Python::with_gil(|py| {
        let locals = get_locals(py, None);
        let data = py.eval("map(list, zip(dates, amounts))", Some(locals), None).unwrap();
        pyxirr_call!(py, "xirr", (data,))
    });
    assert_almost_eq!(result, xirr_expected_result(INPUT));
}

#[rstest]
fn test_extract_from_dict() {
    let input = "tests/samples/unordered.csv";
    let result: f64 = Python::with_gil(|py| {
        let data = PaymentsLoader::from_csv(py, input).to_dict();
        pyxirr_call!(py, "xirr", (data,))
    });
    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_numpy_object_array() {
    let result: f64 = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["numpy"]));
        let data = py.eval("numpy.array([dates, amounts])", Some(locals), None).unwrap();
        pyxirr_call!(py, "xirr", (data,))
    });

    assert_almost_eq!(result, xirr_expected_result(INPUT));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_numpy_arrays() {
    let input = "tests/samples/unordered.csv";
    let result: f64 = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["numpy"]));
        let dates = py.eval("numpy.array(dates)", Some(locals), None).unwrap();
        let amounts = py.eval("numpy.array(amounts)", Some(locals), None).unwrap();
        pyxirr_call!(py, "xirr", (dates, amounts))
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_pandas_dataframe() {
    let input = "tests/samples/unordered.csv";
    let result: f64 = Python::with_gil(|py| {
        let data = pd_read_csv(py, input);
        pyxirr_call!(py, "xirr", (data,))
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_pandas_series() {
    let input = "tests/samples/unordered.csv";
    let result: f64 = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["pandas"]));
        let dates = py.eval("pandas.Series(dates)", Some(locals), None).unwrap();
        let amounts = py.eval("pandas.Series(amounts)", Some(locals), None).unwrap();
        pyxirr_call!(py, "xirr", (dates, amounts))
    });

    assert_almost_eq!(result, xirr_expected_result(input));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_pandas_series_with_datetime_index() {
    let result: f64 = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["pandas"]));
        let dates = py
            .eval("pandas.Series(amounts, index=pandas.to_datetime(dates))", Some(locals), None)
            .unwrap();
        pyxirr_call!(py, "xirr", (dates,))
    });

    assert_almost_eq!(result, xirr_expected_result(INPUT));
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_failed_extract_from_pandas_series_with_int64_index() {
    Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["pandas"]));
        let dates = py.eval("pandas.Series(amounts)", Some(locals), None).unwrap();
        let err = pyxirr_call_impl!(py, "xirr", (dates,)).unwrap_err();
        assert!(err.is_instance::<exceptions::PyTypeError>(py));
    });
}

#[rstest]
#[cfg_attr(feature = "nonumpy", ignore)]
fn test_extract_from_mixed_iterables() {
    let result: f64 = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["pandas", "numpy"]));
        let dates = py.eval("map(pandas.Timestamp, dates)", Some(locals), None).unwrap();
        let amounts = py.eval("numpy.array(amounts)", Some(locals), None).unwrap();
        pyxirr_call!(py, "xirr", (dates, amounts))
    });

    assert_almost_eq!(result, xirr_expected_result(INPUT));
}

#[rstest]
fn test_extract_from_non_float() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let expected = xirr_expected_result(INPUT);

    let locals = get_locals(py, Some(&["decimal"]));
    let dates = locals.get_item("dates").unwrap();

    let amounts = py.eval("map(decimal.Decimal, amounts)", Some(locals), None).unwrap();
    let result: f64 = pyxirr_call!(py, "xirr", (dates, amounts));
    assert_almost_eq!(result, expected);

    let amounts = py.eval("map(int, amounts)", Some(locals), None).unwrap();
    let result: f64 = pyxirr_call!(py, "xirr", (dates, amounts));
    assert_almost_eq!(result, expected);

    let amounts = py.eval("map(str, amounts)", Some(locals), None).unwrap();
    let err = pyxirr_call_impl!(py, "xirr", (dates, amounts)).unwrap_err();
    assert!(err.is_instance::<exceptions::PyTypeError>(py));
}

#[rstest]
fn test_payments_different_sign() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let locals = get_locals(py, None);
    let dates = locals.get_item("dates").unwrap();

    let amounts = py.eval("(abs(x) for x in amounts)", Some(locals), None).unwrap();
    let err = pyxirr_call_impl!(py, "xirr", (dates, amounts)).unwrap_err();
    assert!(err.is_instance::<pyxirr::InvalidPaymentsError>(py));

    let amounts = py.eval("(-abs(x) for x in amounts)", Some(locals), None).unwrap();
    let err = pyxirr_call_impl!(py, "xirr", (dates, amounts)).unwrap_err();
    assert!(err.is_instance::<pyxirr::InvalidPaymentsError>(py));
}

#[rstest]
fn test_arrays_of_dirrerent_lengths() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let locals = get_locals(py, None);
    let dates = locals.get_item("dates").unwrap();
    let amounts = py.eval("amounts[:-2]", Some(locals), None).unwrap();
    let err = pyxirr_call_impl!(py, "xirr", (dates, amounts)).unwrap_err();
    assert!(err.is_instance::<pyxirr::InvalidPaymentsError>(py));
}
