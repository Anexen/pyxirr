use criterion::{black_box, criterion_group, criterion_main, Criterion};

use pyo3::{
    types::{PyDate, PyList},
    Python,
};

use pyxirr;

pub fn criterion_benchmark(c: &mut Criterion) {
    let gil = Python::acquire_gil();
    let py = gil.python();
    // let locals = PyDict::new(py);

    let dates = PyList::new(
        py,
        vec![PyDate::new(py, 2020, 1, 1).unwrap(), PyDate::new(py, 2020, 2, 1).unwrap()],
    );
    let amounts = PyList::new(py, vec![-100, 125]);

    c.bench_function("Simple", |b| {
        b.iter(|| pyxirr::xirr(py, black_box(dates), black_box(Some(amounts)), None))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
