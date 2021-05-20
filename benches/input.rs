#![feature(test)]

extern crate test;

use test::{black_box, Bencher};

use pyo3::Python;

#[path = "../tests/common/mod.rs"]
mod common;

#[bench]
fn bench_from_records(b: &mut Bencher) {
    Python::with_gil(|py| {
        let input = "tests/samples/random_100.csv";
        let data = common::PaymentsLoader::from_csv(py, input).to_records();
        b.iter(|| pyxirr::xirr(black_box(data), black_box(None), black_box(None)).unwrap());
    });
}

#[bench]
fn bench_from_columns(b: &mut Bencher) {
    Python::with_gil(|py| {
        let input = "tests/samples/random_100.csv";
        let data = common::PaymentsLoader::from_csv(py, input).to_columns();
        b.iter(|| {
            pyxirr::xirr(black_box(data.0), black_box(Some(data.1)), black_box(None)).unwrap()
        });
    });
}

#[bench]
fn bench_from_dict(b: &mut Bencher) {
    Python::with_gil(|py| {
        let input = "tests/samples/random_100.csv";
        let data = common::PaymentsLoader::from_csv(py, input).to_dict();
        b.iter(|| pyxirr::xirr(black_box(data), black_box(None), black_box(None)).unwrap());
    });
}

#[bench]
fn bench_from_pandas(b: &mut Bencher) {
    Python::with_gil(|py| {
        let input = "tests/samples/random_100.csv";
        let data = common::pd_read_csv(py, input);
        b.iter(|| pyxirr::xirr(black_box(data), black_box(None), black_box(None)).unwrap());
    });
}
