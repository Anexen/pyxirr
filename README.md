# PyXIRR

Rust-powered collection of financial functions.

# Installation

```
pip install pyxirr
```

# Usage

## xirr

Function signature:

```python
# You have two options:
# 1. Two iterables for dates and amounts
# 2. Single iterable of tuples (date, amount)

DateLike = Union[datetime.date, datetime.datetime]
Amount = Union[int, foat, Decimal]

def xirr(
    dates: Union[Iterable[DateLike], Iterable[Tuple[DateLike, Amount]]]
    amounts Optional[Iterable[Amount]] = None
    guess: Optional[float] = None
)
```

Example:

```python
from datetime import date
from pyxirr import xirr

dates = [date(2020, 1, 1), date(2020, 2, 1)]
amounts = [-100, 125]

xirr(dates, amounts)

# list of tuples is also possible:
xirr(zip(dates, amounts))
```

## xnpv

Function signature:

```python
# similar to xirr: iterable of tuples or two iterables

def xnpv(
    rate: float,
    dates: Union[Iterable[DateLike], Iterable[Tuple[DateLike, Amount]]]
    amounts Optional[Iterable[Amount]] = None
)

```

Example:

```python
xnpv(0.1, dates, amounts)
xnpv(0.1, zip(dates, amounts))
```
