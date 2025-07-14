#![feature(test)]

extern crate test;

use pyo3::{
    ffi::c_str,
    types::{PyAnyMethods, PyModule},
    Python,
};
use std::ffi::CStr;
use test::{black_box, Bencher};

#[path = "../tests/common/mod.rs"]
mod common;
use common::PaymentsLoader;

// https://stackoverflow.com/questions/8919718/financial-python-library-that-has-xirr-and-xnpv-function
const PURE_PYTHON_IMPL: &CStr = c_str!(
    r#"
def xirr(transactions):
    years = [(ta[0] - transactions[0][0]).days / 365.0 for ta in transactions]
    residual = 1
    step = 0.05
    guess = 0.05
    epsilon = 0.0001
    limit = 10000
    while abs(residual) > epsilon and limit > 0:
        limit -= 1
        residual = 0.0
        for i, ta in enumerate(transactions):
            residual += ta[1] / pow(guess + 1, years[i])
        if abs(residual) > epsilon:
            if residual > 0:
                guess += step
            else:
                guess -= step
                step /= 2.0
    return guess
"#
);

const SCIPY_IMPL: &CStr = c_str!(
    r#"
import scipy.optimize

def xnpv(rate, values, dates):
    if rate <= -1.0:
        return float('inf')
    d0 = dates[0]    # or min(dates)
    return sum([ vi / (1.0 + rate)**((di - d0).days / 365.0) for vi, di in zip(values, dates)])

def xirr(values, dates):
    try:
        return scipy.optimize.newton(lambda r: xnpv(r, values, dates), 0.0)
    except RuntimeError:    # Failed to converge?
        return scipy.optimize.brentq(lambda r: xnpv(r, values, dates), -1.0, 1e10)
"#
);

macro_rules! bench_rust {
    ($name:ident, $file:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            Python::with_gil(|py| {
                let data = PaymentsLoader::from_csv(py, $file).to_records();
                b.iter(|| pyxirr_call_impl!(py, "xirr", black_box((&data,))).unwrap());
            });
        }
    };
}

macro_rules! bench_scipy {
    ($name:ident, $file:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            Python::with_gil(|py| {
                let py_mod =
                    PyModule::from_code(py, SCIPY_IMPL, c_str!("xirr.py"), c_str!("scipy_py_xirr"))
                        .unwrap();

                let xirr = py_mod.getattr("xirr").unwrap();
                let data = PaymentsLoader::from_csv(py, $file).to_columns();
                b.iter(|| xirr.call1(black_box((&data.1, &data.0))).unwrap())
            });
        }
    };
}

macro_rules! bench_python {
    ($name:ident, $file:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            Python::with_gil(|py| {
                let py_mod = PyModule::from_code(
                    py,
                    PURE_PYTHON_IMPL,
                    c_str!("xirr.py"),
                    c_str!("pure_py_xirr"),
                )
                .unwrap();

                let xirr = py_mod.getattr("xirr").unwrap();
                let data = PaymentsLoader::from_csv(py, $file).to_records();
                b.iter(|| xirr.call1(black_box((&data,))).unwrap())
            });
        }
    };
}

bench_rust!(bench_rust_50, "tests/samples/rw-50.csv");
bench_rust!(bench_rust_100, "tests/samples/rw-100.csv");
bench_rust!(bench_rust_500, "tests/samples/rw-500.csv");
bench_rust!(bench_rust_1000, "tests/samples/rw-1000.csv");

bench_scipy!(bench_scipy_50, "tests/samples/rw-50.csv");
bench_scipy!(bench_scipy_100, "tests/samples/rw-100.csv");
bench_scipy!(bench_scipy_500, "tests/samples/rw-500.csv");
bench_scipy!(bench_scipy_1000, "tests/samples/rw-1000.csv");

bench_python!(bench_python_50, "tests/samples/rw-50.csv");
bench_python!(bench_python_100, "tests/samples/rw-100.csv");
bench_python!(bench_python_500, "tests/samples/rw-500.csv");
bench_python!(bench_python_1000, "tests/samples/rw-1000.csv");
