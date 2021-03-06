[![rust-lang.org](https://img.shields.io/badge/Made%20with-Rust-red)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/Anexen/pyxirr.svg)](https://github.com/Anexen/pyxirr/blob/master/LICENSE)
[![pypi](https://img.shields.io/pypi/v/pyxirr.svg)](https://pypi.org/project/pyxirr/)
[![versions](https://img.shields.io/pypi/pyversions/pyxirr.svg)](https://pypi.org/project/pyxirr/)

# PyXIRR

Rust-powered collection of financial functions.

PyXIRR stands for "Python XIRR" (for historical reasons), but contains many other financial functions such as IRR, FV, NPV, etc.

Features:

- correct
- blazingly fast
- works with different input data types (iterators, numpy arrays, pandas DataFrames)
- no external dependencies

# Installation

```
pip install pyxirr
```

# Benchmarks

Rust implementation has been tested against existing [xirr](https://pypi.org/project/xirr/) package
(uses [scipy.optimize](https://docs.scipy.org/doc/scipy/reference/generated/scipy.optimize.newton.html) under the hood)
and the [implementation from the Stack Overflow](https://stackoverflow.com/a/11503492) (pure python).

![bench](https://raw.githubusercontent.com/Anexen/pyxirr/main/docs/static/bench.png)

PyXIRR is ~10-20x faster in XIRR calculation than the other implementations.

Powered by [github-action-benchmark](https://github.com/rhysd/github-action-benchmark) and [plotly.js](https://github.com/plotly/plotly.js).

Live benchmarks are hosted on [Github Pages](https://anexen.github.io/pyxirr/bench).

# Examples

```python
from datetime import date
from pyxirr import xirr

dates = [date(2020, 1, 1), date(2021, 1, 1), date(2022, 1, 1)]
amounts = [-1000, 750, 500]

# feed columnar data
xirr(dates, amounts)
# feed iterators
xirr(iter(dates), (x / 2 for x in amounts))
# feed an iterable of tuples
xirr(zip(dates, amounts))
# feed a dictionary
xirr(dict(zip(dates, amounts)))
# dates as strings
xirr(['2020-01-01', '2021-01-01'], [-1000, 1200])
```

Numpy and Pandas support

```python
import numpy as np
import pandas as pd

# feed numpy array
xirr(np.array([dates, amounts]))
xirr(np.array(dates), np.array(amounts))

# feed DataFrame (columns names doesn't matter; ordering matters)
xirr(pd.DataFrame({"a": dates, "b": amounts}))

# feed Series with DatetimeIndex
xirr(pd.Series(amounts, index=pd.to_datetime(dates)))

# bonus: apply xirr to a DataFrame with DatetimeIndex:
df = pd.DataFrame(
    index=pd.date_range("2021", "2022", freq="MS", closed="left"),
    data={
        "one": [-100] + [20] * 11,
        "two": [-80] + [19] * 11,
    },
)
df.apply(xirr)  # Series(index=["one", "two"], data=[5.09623547168478, 8.780801977141174])
```

Other financial functions:

```python
import pyxirr

# Future Value
pyxirr.fv(0.05 / 12, 10 * 12, -100, -100)

# Net Present Value
pyxirr.npv(0, [-40_000, 5_000, 8_000, 12_000, 30_000])

# IRR
pyxirr.irr([-100, 39, 59, 55, 20])

# ... and more! Check out the docs.
```

# API reference

See the [docs](https://anexen.github.io/pyxirr)

# Roadmap

- [x] Implement all functions from [numpy-financial](https://numpy.org/numpy-financial/latest/index.html)
- [x] Improve docs, add more tests
- [x] Type hints
- [ ] Vectorized versions of numpy-financial functions.
- [ ] Compile library for rust/javascript/python

# Development

Running tests with pyo3 is a bit tricky. In short, you need to compile your tests without `extension-module` feature to avoid linking errors.
See the following issues for the details: [#341](https://github.com/PyO3/pyo3/issues/341), [#771](https://github.com/PyO3/pyo3/issues/771).

If you are using `pyenv`, make sure you have the shared library installed (check for `${PYENV_ROOT}/versions/<version>/lib/libpython3.so` file).

```bash
$ PYTHON_CONFIGURE_OPTS="--enable-shared" pyenv install <version>
```

Install dev-requirements

```bash
$ pip install -r dev-requirements.txt
```

### Building

```bash
$ maturin develop
```

### Testing

```bash
$ LD_LIBRARY_PATH=${PYENV_ROOT}/versions/3.8.6/lib cargo test --no-default-features --features tests
```

### Benchmarks

```bash
$ pip install -r bench-requirements.txt
$ LD_LIBRARY_PATH=${PYENV_ROOT}/versions/3.8.6/lib cargo +nightly bench --no-default-features --features tests
```

# Building and distribution

This library uses [maturin](https://github.com/PyO3/maturin) to build and distribute python wheels.

```bash
$ docker run --rm -v $(pwd):/io konstin2/maturin build --release --manylinux 2010 --strip
$ maturin upload target/wheels/pyxirr-${version}*
```
