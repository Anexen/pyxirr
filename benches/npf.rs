#![feature(test)]

extern crate test;

use pyo3::{types::*, Python};
use test::{black_box, Bencher};

#[path = "../tests/common/mod.rs"]
mod common;

static B_1: &[i32] = &[-100, 39, 59, 55, 20];

#[bench]
fn bench_1_irr(b: &mut Bencher) {
    Python::with_gil(|py| {
        let payments = PyList::new(py, B_1);
        b.iter(|| black_box(pyxirr_call_impl!(py, "irr", (payments,)).unwrap()));
    });
}

#[bench]
fn bench_1_irr_npf(b: &mut Bencher) {
    Python::with_gil(|py| {
        let irr = py.import("numpy_financial").unwrap().getattr("irr").unwrap();
        let payments = PyList::new(py, B_1);
        b.iter(|| black_box(irr.call1((payments,)).unwrap()))
    });
}

static B_2: &[f64] = &[
    -217500.0,
    -217500.0,
    108466.80462450592,
    101129.96439328062,
    93793.12416205535,
    86456.28393083003,
    79119.44369960476,
    71782.60346837944,
    64445.76323715414,
    57108.92300592884,
    49772.08277470355,
    42435.24254347826,
    35098.40231225296,
    27761.56208102766,
    20424.721849802358,
    13087.88161857707,
    5751.041387351768,
    -1585.7988438735192,
    -8922.639075098821,
    -16259.479306324123,
    -23596.31953754941,
    -30933.159768774713,
    -38270.0,
    -45606.8402312253,
    -52943.680462450604,
    -60280.520693675906,
    -67617.36092490121,
];

#[bench]
fn bench_2_irr(b: &mut Bencher) {
    Python::with_gil(|py| {
        let payments = PyList::new(py, B_2);
        b.iter(|| black_box(pyxirr_call_impl!(py, "irr", (payments,)).unwrap()));
    });
}

#[bench]
fn bench_2_irr_npf(b: &mut Bencher) {
    Python::with_gil(|py| {
        let irr = py.import("numpy_financial").unwrap().getattr("irr").unwrap();
        let payments = PyList::new(py, B_2);
        b.iter(|| black_box(irr.call1((payments,)).unwrap()))
    });
}

static B_3: &[f64] = &[10.0, 1.0, 2.0, -3.0, 4.0];

#[bench]
fn bench_3_irr_none(b: &mut Bencher) {
    Python::with_gil(|py| {
        let payments = PyList::new(py, B_3);
        b.iter(|| black_box(pyxirr_call_impl!(py, "irr", (payments,)).unwrap()));
    });
}

#[bench]
fn bench_3_irr_none_npf(b: &mut Bencher) {
    Python::with_gil(|py| {
        let irr = py.import("numpy_financial").unwrap().getattr("irr").unwrap();
        let payments = PyList::new(py, B_3);
        b.iter(|| black_box(irr.call1((payments,)).unwrap()))
    });
}

static B_4: &[i32] = &[-1000, 100, 250, 500, 500];

#[bench]
fn bench_4_mirr(b: &mut Bencher) {
    Python::with_gil(|py| {
        let values = PyList::new(py, B_4);
        b.iter(|| black_box(pyxirr_call_impl!(py, "mirr", (values, 0.1, 0.1)).unwrap()))
    });
}

#[bench]
fn bench_4_mirr_npf(b: &mut Bencher) {
    Python::with_gil(|py| {
        let mirr = py.import("numpy_financial").unwrap().getattr("mirr").unwrap();
        let values = PyList::new(py, B_4);
        b.iter(|| black_box(mirr.call1((values, 0.1, 0.1)).unwrap()))
    });
}
