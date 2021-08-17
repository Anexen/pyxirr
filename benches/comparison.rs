#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

use pyo3::{types::PyModule, Python};

#[path = "../tests/common/mod.rs"]
mod common;
use common::PaymentsLoader;

// https://stackoverflow.com/questions/8919718/financial-python-library-that-has-xirr-and-xnpv-function
const TOP_STACK_OVERFLOW_ANSWER: &str = r#"
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
"#;

macro_rules! bench_rust {
    ($name:ident, $file:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            Python::with_gil(|py| {
                let data = PaymentsLoader::from_csv(py, $file).to_records();
                b.iter(|| pyxirr::xirr(black_box(data), black_box(None), black_box(None)).unwrap());
            });
        }
    };
}

macro_rules! bench_scipy {
    ($name:ident, $file:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            Python::with_gil(|py| {
                let xirr =
                    py.import("xirr").expect("xirr is not installed").getattr("xirr").unwrap();
                let data = PaymentsLoader::from_csv(py, $file).to_dict();
                b.iter(|| xirr.call1(black_box((data,))).unwrap())
            });
        }
    };
}

macro_rules! bench_python {
    ($name:ident, $file:expr) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            Python::with_gil(|py| {
                let xirr =
                    PyModule::from_code(py, TOP_STACK_OVERFLOW_ANSWER, "xirr.py", "pure_py_xirr")
                        .unwrap()
                        .getattr("xirr")
                        .unwrap();
                let data = PaymentsLoader::from_csv(py, $file).to_records();
                b.iter(|| xirr.call1(black_box((data,))).unwrap())
            });
        }
    };
}

bench_rust!(bench_rust_100, "tests/samples/random_100.csv");
bench_rust!(bench_rust_500, "tests/samples/random_500.csv");
bench_rust!(bench_rust_1000, "tests/samples/random_1000.csv");

bench_scipy!(bench_scipy_100, "tests/samples/random_100.csv");
bench_scipy!(bench_scipy_500, "tests/samples/random_500.csv");
bench_scipy!(bench_scipy_1000, "tests/samples/random_1000.csv");

bench_python!(bench_python_100, "tests/samples/random_100.csv");
bench_python!(bench_python_500, "tests/samples/random_500.csv");
bench_python!(bench_python_1000, "tests/samples/random_1000.csv");
