#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

use pyo3::types::*;
use pyo3::Python;

#[path = "../tests/common/mod.rs"]
mod common;

#[bench]
fn bench_irr(b: &mut Bencher) {
    Python::with_gil(|py| {
        let payments = PyList::new(py, &[-100, 39, 59, 55, 20]);
        b.iter(|| pyxirr_call_impl!(py, "irr", (payments,)).unwrap());
    });
}

#[bench]
fn bench_irr_npf(b: &mut Bencher) {
    Python::with_gil(|py| {
        let irr = py.import("numpy_financial").unwrap().getattr("irr").unwrap();
        let payments = PyList::new(py, &[-100, 39, 59, 55, 20]);
        b.iter(|| irr.call1(black_box((payments,))).unwrap())
    });
}

#[bench]
fn bench_mirr(b: &mut Bencher) {
    Python::with_gil(|py| {
        let values = PyList::new(py, &[-1000, 100, 250, 500, 500]);
        b.iter(|| pyxirr_call_impl!(py, "mirr", (values, 0.1, 0.1)).unwrap())
    });
}

#[bench]
fn bench_mirr_npf(b: &mut Bencher) {
    Python::with_gil(|py| {
        let mirr = py.import("numpy_financial").unwrap().getattr("mirr").unwrap();
        let values = PyList::new(py, &[-1000, 100, 250, 500, 500]);
        b.iter(|| mirr.call1(black_box((values, 0.1, 0.1))).unwrap())
    });
}
