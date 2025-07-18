#![allow(dead_code)]
use pyo3::{ffi::c_str, prelude::*, sync::GILOnceCell, types::*};

#[macro_export]
macro_rules! py_dict {
    ($py:expr) => {
        ::pyo3::types::PyDict::new($py)
    };
    ($py:expr, $($key:expr => $value:expr), *) => {
        {
            use pyo3::prelude::PyDictMethods;
            let _dict = ::pyo3::types::PyDict::new($py);
            $(
                _dict.set_item($key, $value).unwrap();
            )*
            _dict
        }
    };
}

#[macro_export]
macro_rules! py_dict_merge {
    ($py:expr, $($dict:expr), *) => {
        {
            let _dict = ::pyo3::types::PyDict::new($py);
            $(
                _dict.getattr("update").unwrap().call1(($dict,)).unwrap();
            )*
            _dict

        }
    }
}

#[macro_export]
macro_rules! pyxirr_call {
    ($py:expr, $name:expr, $args:expr) => {{
        let kwargs = ::pyo3::types::PyDict::new($py);
        pyxirr_call!($py, $name, $args, kwargs)
    }};
    ($py:expr, $name:expr, $args:expr, $kwargs:expr) => {{
        use pyo3::prelude::PyAnyMethods;
        pyxirr_call_impl!($py, $name, $args, $kwargs).unwrap().extract().unwrap()
    }};
}

#[macro_export]
macro_rules! pyxirr_call_impl {
    ($py:expr, $name:expr, $args:expr) => {{
        let kwargs = ::pyo3::types::PyDict::new($py);
        pyxirr_call_impl!($py, $name, $args, kwargs)
    }};
    ($py:expr, $name:expr, $args:expr, $kwargs:expr) => {{
        use common::get_pyxirr_func;
        use pyo3::prelude::PyAnyMethods;
        get_pyxirr_func($py, $name).call($args, Some(&$kwargs))
    }};
}

#[macro_export]
macro_rules! assert_almost_eq {
    ($a:expr, $b:expr, $eps:expr) => {{
        let (a, b, eps) = (&$a, &$b, $eps);
        assert!((*a - *b).abs() < eps, "assertion failed: `({} !~= {})`", *a, *b);
    }};
    ($a:expr, $b:expr) => {{
        let (a, b) = (&$a, &$b);
        let eps: f64 = 1e-9;
        assert!((*a - *b).abs() < eps, "assertion failed: `({} !~= {})`", *a, *b);
    }};
}

#[macro_export]
macro_rules! assert_future_value {
    ($rate:expr, $nper:expr, $pmt:expr, $pv:expr, $fv:expr, $pmt_at_beginning:expr) => {{
        let (rate, nper, pmt, pv, fv, pmt_at_beginning) =
            ($rate, $nper, $pmt, $pv, $fv, $pmt_at_beginning);

        let fv = fv.unwrap_or(0.0);

        if rate == 0.0 {
            assert_almost_eq!(fv + pv + pmt * nper, 0.0);
            return;
        }

        let pmt_at_beginning = if pmt_at_beginning.unwrap_or(false) {
            1.0
        } else {
            0.0
        };

        let result = fv
            + pv * f64::powf(1.0 + rate, nper)
            + pmt * (1.0 + rate * pmt_at_beginning) / rate * (f64::powf(1.0 + rate, nper) - 1.0);

        assert_almost_eq!(result, 0.0, 1e-6);
    }};
}

static PYXIRR: GILOnceCell<Py<PyModule>> = GILOnceCell::new();

pub fn get_pyxirr_module(py: Python<'_>) -> &Bound<PyModule> {
    PYXIRR
        .get_or_init(py, || {
            let module = PyModule::new(py, "pyxirr").unwrap();
            pyxirr::pyxirr(py, &module).unwrap();
            module.into()
        })
        .bind(py)
}

pub fn get_pyxirr_func<'p>(py: Python<'p>, name: &str) -> Bound<'p, PyCFunction> {
    get_pyxirr_module(py).getattr(name).unwrap().downcast_into().unwrap()
}

pub fn pd_read_csv<'p>(py: Python<'p>, input_file: &str) -> Bound<'p, PyAny> {
    let locals = py_dict!(py,
        "sample" => PyString::new(py, input_file),
        "pd" => PyModule::import(py, "pandas").unwrap()
    );

    py.eval(c_str!("pd.read_csv(sample, header=None, parse_dates=[0])"), Some(&locals), None)
        .unwrap()
}

pub struct PaymentsLoader<'p> {
    py: Python<'p>,
    data: Vec<Bound<'p, PyTuple>>,
}

impl<'p> PaymentsLoader<'p> {
    pub fn from_csv(py: Python<'p>, input_file: &str) -> Self {
        let data = Self::from_py_csv(py, input_file).unwrap();
        Self {
            py,
            data,
        }
    }

    fn from_py_csv(py: Python<'p>, input_file: &str) -> PyResult<Vec<Bound<'p, PyTuple>>> {
        let strptime = py.import("datetime")?.getattr("datetime")?.getattr("strptime")?;
        let reader = py.import("csv")?.getattr("reader")?;
        let builtins = py.import("builtins")?;
        let file_obj = builtins.getattr("open")?.call1((input_file,))?;

        let data = reader
            .call1((file_obj.clone(),))?
            .try_iter()?
            .map(|r| {
                let r = r.unwrap();
                let date = strptime.call1((r.get_item(0)?, "%Y-%m-%d"))?;
                let amount = builtins.getattr("float")?.call1((r.get_item(1)?,))?;
                Ok(PyTuple::new(py, vec![date, amount]).unwrap())
            })
            .collect();

        file_obj.call_method0("close")?;

        data
    }

    pub fn to_records(&self) -> Bound<'p, PyAny> {
        PyList::new(self.py, &self.data).unwrap().into_any()
    }

    pub fn to_dict(&self) -> Bound<'p, PyAny> {
        PyDict::from_sequence(&self.to_records()).unwrap().into_any()
    }

    pub fn to_columns(&self) -> (Bound<'p, PyAny>, Bound<'p, PyAny>) {
        (
            PyList::new(self.py, self.data.iter().map(|x| x.get_item(0).unwrap()))
                .unwrap()
                .into_any(),
            PyList::new(self.py, self.data.iter().map(|x| x.get_item(1).unwrap()))
                .unwrap()
                .into_any(),
        )
    }
}
