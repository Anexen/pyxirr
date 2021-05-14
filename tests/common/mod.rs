#![allow(dead_code)]

use pyo3::prelude::*;
use pyo3::types::{PyFloat, PyList, PyTuple};

#[macro_export]
macro_rules! assert_almost_eq {
    ($a:expr, $b:expr) => {{
        let (a, b) = (&$a, &$b);
        let eps: f64 = 1e-10;
        assert!((*a - *b).abs() < eps, "assertion failed: `({} !~= {})`", *a, *b);
    }};
}

pub fn load_payments<'p>(
    py: Python<'p>,
    file: &str,
    columnar: bool,
) -> (&'p PyAny, Option<&'p PyAny>) {
    let strptime = PyModule::import(py, "datetime")
        .unwrap()
        .getattr("datetime")
        .unwrap()
        .getattr("strptime")
        .unwrap();

    let data = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(file)
        .unwrap()
        .records()
        .map(|r| r.unwrap())
        .map(|r| {
            let date = strptime.call1((&r[0], "%Y-%m-%d")).unwrap();
            let amount = PyFloat::new(py, r[1].parse::<f64>().unwrap()).as_ref();
            PyTuple::new(py, vec![date, amount]).as_ref()
        })
        .collect::<Vec<&PyAny>>();

    if columnar {
        let dates =
            PyList::new(py, data.iter().map(|x| x.get_item(0).unwrap()).collect::<Vec<&PyAny>>());
        let amounts =
            PyList::new(py, data.iter().map(|x| x.get_item(1).unwrap()).collect::<Vec<&PyAny>>());

        (dates, Some(amounts))
    } else {
        (PyList::new(py, data).as_ref(), None)
    }
}
