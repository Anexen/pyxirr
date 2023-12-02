[![rust-lang.org](https://img.shields.io/badge/Made%20with-Rust-red)](https://www.rust-lang.org/)
[![License](https://img.shields.io/github/license/Anexen/pyxirr.svg)](https://github.com/Anexen/pyxirr/blob/master/LICENSE)
[![pypi](https://img.shields.io/pypi/v/pyxirr.svg)](https://pypi.org/project/pyxirr/)
[![versions](https://img.shields.io/pypi/pyversions/pyxirr.svg)](https://pypi.org/project/pyxirr/)

# PyXIRR

Rust-powered collection of financial functions.

PyXIRR stands for "Python XIRR" (for historical reasons), but contains many other financial functions such as IRR, FV, NPV, etc.

Features:

- correct
- supports different day count conventions (e.g. ACT/360, 30E/360, etc.)
- works with different input data types (iterators, numpy arrays, pandas DataFrames)
- no external dependencies
- type annotations
- blazingly fast

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

# Example

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

# Multiple IRR problem

The Multiple IRR problem occur when the signs of cash flows change more than
once. In this case, we say that the project has non-conventional cash flows.
This leads to situation, where it can have more the one IRR or have no IRR at all.

PyXIRR's approach to the Multiple IRR problem:

1. It looks for positive result around 0.1 (the same as Excel with the default guess=0.1).
2. If it can't find a result, it uses several other attempts and selects the lowest IRR to be conservative.

Here is an example of how to find multiple IRRs:

```python
import numpy as np
import pyxirr

# load cash flow:
cf = pd.read_csv("tests/samples/30-22.csv", names=["date", "amount"])
# check whether the cash flow is conventional:
print(pyxirr.is_conventional_cash_flow(cf["amount"]))  # false

# build NPV profile:
# calculate 50 NPV values for different rates
rates = np.linspace(-0.5, 0.5, 50)
# any iterable, any rates, e.g.
# rates = [-0.5, -0.3, -0.1, 0.1, -0.6]
values = pyxirr.xnpv(rates, cf)

# print NPV profile:
# NPV changes sign two times:
#   1) between -0.316 and -0.295
#   2) between -0.03 and -0.01
print("NPV profile:")
for rate, value in zip(rates, values):
    print(rate, value)

# plot NPV profile
import pandas as pd
series = pd.Series(values, index=rates)
pd.DataFrame(series[series > -1e6]).assign(zero=0).plot()

# find points where NPV function crosses zero
indexes = pyxirr.zero_crossing_points(values)

print("Zero crossing points:")
for idx in indexes:
    print("between", rates[idx], "and", rates[idx+1])

# XIRR has two results:
#   -0.31540826742734207
#   -0.028668460065441048
for i, idx in enumerate(indexes, start=1):
    rate = pyxirr.xirr(cf, guess=rates[idx])
    npv = pyxirr.xnpv(rate, cf)
    print(f"{i}) {rate}; XNPV = {npv}")
```

# More Examples

### Numpy and Pandas

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
    index=pd.date_range("2021", "2022", freq="MS", inclusive="left"),
    data={
        "one": [-100] + [20] * 11,
        "two": [-80] + [19] * 11,
    },
)
df.apply(xirr)  # Series(index=["one", "two"], data=[5.09623547168478, 8.780801977141174])
```

### Day count conventions

Check out the available options on the [docs/day-count-conventions](https://anexen.github.io/pyxirr/functions.html#day-count-conventions).

```python
from pyxirr import DayCount

xirr(dates, amounts, day_count=DayCount.ACT_360)

# parse day count from string
xirr(dates, amounts, day_count="30E/360")
```

### Private equity performance metrics

```python
from pyxirr import pe

pe.pme_plus([-20, 15, 0], index=[100, 115, 130], nav=20)

pe.direct_alpha([-20, 15, 0], index=[100, 115, 130], nav=20)
```

[Docs](https://anexen.github.io/pyxirr/private_equity.html)

### Other financial functions

```python
import pyxirr

# Future Value
pyxirr.fv(0.05/12, 10*12, -100, -100)

# Net Present Value
pyxirr.npv(0, [-40_000, 5_000, 8_000, 12_000, 30_000])

# IRR
pyxirr.irr([-100, 39, 59, 55, 20])

# ... and more! Check out the docs.
```

[Docs](https://anexen.github.io/pyxirr/functions.html)

### Vectorization

PyXIRR supports numpy-like vectorization.

If all input is scalar, returns a scalar float. If any input is array_like,
returns values for each input element. If multiple inputs are
array_like, performs broadcasting and returns values for each element.

```python
import pyxirr

# feed list
pyxirr.fv([0.05/12, 0.06/12], 10*12, -100, -100)
pyxirr.fv([0.05/12, 0.06/12], [10*12, 9*12], [-100, -200], -100)

# feed numpy array
import numpy as np
rates = np.array([0.05, 0.06, 0.07])/12
pyxirr.fv(rates, 10*12, -100, -100)

# feed any iterable!
pyxirr.fv(
    np.linspace(0.01, 0.2, 10),
    (x + 1 for x in range(10)),
    range(-100, -1100, -100),
    tuple(range(-100, -200, -10))
)

# 2d, 3d, 4d, and more!
rates = [[[[[[0.01], [0.02]]]]]]
pyxirr.fv(rates, 10*12, -100, -100)
```

# API reference

See the [docs](https://anexen.github.io/pyxirr)

# Roadmap

- [x] Implement all functions from [numpy-financial](https://numpy.org/numpy-financial/latest/index.html)
- [x] Improve docs, add more tests
- [x] Type hints
- [x] Vectorized versions of numpy-financial functions.
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
$ LD_LIBRARY_PATH=${PYENV_ROOT}/versions/3.10.8/lib cargo test
```

### Benchmarks

```bash
$ pip install -r bench-requirements.txt
$ LD_LIBRARY_PATH=${PYENV_ROOT}/versions/3.10.8/lib cargo +nightly bench
```

# Building and distribution

This library uses [maturin](https://github.com/PyO3/maturin) to build and distribute python wheels.

```bash
$ docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin build --release --manylinux 2010 --strip
$ maturin upload target/wheels/pyxirr-${version}*
```
