use pyo3::prelude::*;
use pyo3::types::*;
use rstest::*;

mod common;

use pyxirr;

// -------------------- TEST DATA -----------------------------------

const EXPECTED: f64 = 0.6164943046;

fn get_dates(py: Python) -> &PyList {
    PyList::new(
        py,
        vec![
            PyDate::new(py, 2020, 1, 1).unwrap().as_ref(),
            PyDate::new(py, 2021, 1, 1).unwrap().as_ref(),
            PyDate::new(py, 2022, 1, 1).unwrap().as_ref(),
        ],
    )
}

fn get_amounts(py: Python) -> &PyList {
    PyList::new(
        py,
        vec![
            PyFloat::new(py, -1000.0).as_ref(),
            PyFloat::new(py, 1000.0).as_ref(),
            PyFloat::new(py, 1000.0).as_ref(),
        ],
    )
}

// ------------------------------------------------------------------

fn get_locals<'p>(py: Python<'p>, extra_imports: Option<&[&str]>) -> &'p PyDict {
    let builtins = PyModule::import(py, "builtins").unwrap();

    let mut locals = vec![
        ("amounts", get_amounts(py).as_ref()),
        ("dates", get_dates(py).as_ref()),
        ("iter", builtins.getattr("iter").unwrap()),
        ("list", builtins.getattr("list").unwrap()),
        ("zip", builtins.getattr("zip").unwrap()),
        ("map", builtins.getattr("map").unwrap()),
    ];

    for &name in extra_imports.unwrap_or_default() {
        let module = PyModule::import(py, name).expect(&format!("{:?} is not installed", name));
        locals.push((name, module))
    }

    locals.into_py_dict(py)
}

#[rstest]
fn test_extract_from_iter() {
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, None);

        let dates_iter = py.eval("iter(dates)", Some(locals), None).unwrap();
        let amounts_gen = py.eval("(x for x in amounts)", Some(locals), None).unwrap();

        pyxirr::xirr(dates_iter, Some(amounts_gen), None)
    });

    assert_almost_eq!(result.unwrap(), EXPECTED);
}

#[rstest]
fn test_extract_from_tuples() {
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, None);
        let cash_flow = py.eval("zip(dates, amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(cash_flow, None, None)
    });
    assert_almost_eq!(result.unwrap(), EXPECTED);
}

#[rstest]
fn test_extract_from_lists() {
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, None);
        let cash_flow = py.eval("map(list, zip(dates, amounts))", Some(locals), None).unwrap();
        pyxirr::xirr(cash_flow, None, None)
    });
    assert_almost_eq!(result.unwrap(), EXPECTED);
}

#[rstest]
fn test_extract_from_dict() {
    let result = Python::with_gil(|py| {
        let data = get_dates(py).iter().zip(get_amounts(py)).into_py_dict(py);
        pyxirr::xirr(data, None, None)
    });
    assert_almost_eq!(result.unwrap(), EXPECTED);
}

#[rstest]
fn test_extract_from_numpy_object_array() {
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["numpy"]));
        let data = py.eval("numpy.array([dates, amounts])", Some(locals), None).unwrap();
        pyxirr::xirr(data, None, None)
    });

    assert_almost_eq!(result.unwrap(), EXPECTED);
}

#[rstest]
fn test_extract_from_numpy_arrays() {
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["numpy"]));
        let dates = py.eval("numpy.array(dates)", Some(locals), None).unwrap();
        let amounts = py.eval("numpy.array(amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None)
    });

    assert_almost_eq!(result.unwrap(), EXPECTED);
}

#[rstest]
fn test_extract_from_pandas_dataframe() {
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["pandas"]));
        let data = py
            .eval("pandas.DataFrame({'dates': dates, 'amounts': amounts})", Some(locals), None)
            .unwrap();
        pyxirr::xirr(data, None, None)
    });

    assert_almost_eq!(result.unwrap(), EXPECTED);
}

#[rstest]
fn test_extract_from_pandas_series() {
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["pandas"]));
        let dates = py.eval("pandas.Series(dates)", Some(locals), None).unwrap();
        let amounts = py.eval("pandas.Series(amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None)
    });

    assert_almost_eq!(result.unwrap(), EXPECTED);
}

#[rstest]
fn test_extract_from_mixed_iterables() {
    let result = Python::with_gil(|py| {
        let locals = get_locals(py, Some(&["pandas", "numpy"]));
        let dates = py.eval("map(pandas.Timestamp, dates)", Some(locals), None).unwrap();
        let amounts = py.eval("numpy.array(amounts)", Some(locals), None).unwrap();
        pyxirr::xirr(dates, Some(amounts), None)
    });

    assert_almost_eq!(result.unwrap(), EXPECTED);
}

#[rstest]
#[case("tests/samples/unordered.csv", 0.16353715844)]
#[case("tests/samples/random_100.csv", 29.829404437653)]
#[case("tests/samples/random_1000.csv", 5.508930558032)]
#[case("tests/samples/random_10000.csv", 0.350185149995)]
fn test_pandas_read_csv(#[case] input: &str, #[case] expected: f64) {
    let result = Python::with_gil(|py| {
        let locals = vec![
            ("sample", PyString::new(py, input).as_ref()),
            ("pd", PyModule::import(py, "pandas").unwrap()),
        ]
        .into_py_dict(py);

        let data = py
            .eval("pd.read_csv(sample, header=None, parse_dates=[0])", Some(locals), None)
            .unwrap();
        pyxirr::xirr(data, None, None)
    });

    assert_almost_eq!(result.unwrap(), expected);
}
