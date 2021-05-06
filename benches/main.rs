use chrono::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyAny, PyDate, PyDict, PyFloat, PyList, PyTuple};

use pyxirr;

fn generate_payments<'a>(py: Python<'a>) -> &'a PyList {
    let dates = (0..10000).map(move |i| PyDate::from_timestamp(py, i * 24 * 60 * 60).unwrap());
    let data = dates.zip(-2000..8000).collect::<Vec<(&PyDate, i64)>>();
    return PyList::new(py, data);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let data = generate_payments(py);

    let xirr = py.import("xirr").expect("xirr is not installed").getattr("xirr").unwrap();
    // let fc = py.import("finance_calculator").expect("finance_calculator is not installed").getattr("get_xirr").unwrap();

    let mut group = c.benchmark_group("XIRR");

    group.bench_function(BenchmarkId::new("pyxirr", ""), |b| {
        b.iter(|| pyxirr::faster_xirr(py, black_box(data), None))
    });

    group.bench_function(BenchmarkId::new("xirr", ""), |b| {
        let xdata = PyDict::from_sequence(py, data.into()).unwrap();
        b.iter(|| xirr.call1(black_box((xdata,))).unwrap())
    });

    // group.bench_function(BenchmarkId::new("fc", ""), |b| {
    //     b.iter(|| fc.call1(black_box((data,))).unwrap())
    // });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
