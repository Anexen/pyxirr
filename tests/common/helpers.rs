#![allow(dead_code)]
use pyo3::prelude::*;
use pyo3::types::*;

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
    ($rate:expr, $nper:expr, $pmt:expr, $pv:expr, $fv:expr, $pmt_at_begining:expr) => {{
        let (rate, nper, pmt, pv, fv, pmt_at_begining) =
            ($rate, $nper, $pmt, $pv, $fv, $pmt_at_begining);

        let fv = fv.unwrap_or(0.0);

        if rate == 0.0 {
            assert_almost_eq!(fv + pv + pmt * nper, 0.0);
            return;
        }

        let pmt_at_begining = if pmt_at_begining.unwrap_or(false) { 1.0 } else { 0.0 };

        let result = fv
            + pv * f64::powf(1.0 + rate, nper)
            + pmt * (1.0 + rate * pmt_at_begining) / rate * (f64::powf(1.0 + rate, nper) - 1.0);

        assert_almost_eq!(result, 0.0, 1e-6);
    }};
}

pub fn pd_read_csv<'p>(py: Python<'p>, input_file: &str) -> &'p PyAny {
    let locals = vec![
        ("sample", PyString::new(py, input_file).as_ref()),
        ("pd", PyModule::import(py, "pandas").unwrap()),
    ]
    .into_py_dict(py);

    py.eval("pd.read_csv(sample, header=None, parse_dates=[0])", Some(locals), None).unwrap()
}

pub struct PaymentsLoader<'p> {
    py: Python<'p>,
    data: Vec<&'p PyTuple>,
}

impl<'p> PaymentsLoader<'p> {
    pub fn from_csv(py: Python<'p>, input_file: &str) -> Self {
        let data = Self::from_py_csv(py, input_file).unwrap();
        Self { py, data }
    }

    fn from_py_csv(py: Python<'p>, input_file: &str) -> PyResult<Vec<&'p PyTuple>> {
        let strptime = py.import("datetime")?.getattr("datetime")?.getattr("strptime")?;
        let reader = py.import("csv")?.getattr("reader")?;
        let builtins = py.import("builtins")?;
        let file_obj = builtins.getattr("open")?.call1((input_file,))?;

        let data = reader
            .call1((file_obj,))?
            .iter()?
            .map(|r| {
                let r = r.unwrap();
                let date = strptime.call1((r.get_item(0)?, "%Y-%m-%d"))?;
                let amount = builtins.getattr("float")?.call1((r.get_item(1)?,))?;
                Ok(PyTuple::new(py, vec![date, amount]))
            })
            .collect();

        file_obj.call_method0("close")?;

        data
    }

    pub fn to_records(&self) -> &'p PyAny {
        PyList::new(self.py, &self.data).as_ref()
    }

    pub fn to_dict(&self) -> &'p PyAny {
        PyDict::from_sequence(self.py, self.to_records().into()).unwrap().as_ref()
    }

    pub fn to_columns(&self) -> (&'p PyAny, &'p PyAny) {
        (
            PyList::new(self.py, self.data.iter().map(|x| x.get_item(0))).as_ref(),
            PyList::new(self.py, self.data.iter().map(|x| x.get_item(1))).as_ref(),
        )
    }
}
