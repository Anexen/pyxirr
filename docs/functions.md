{% include head.html %}

# Functions

> By convention, investments or "deposits" are negative, income or "withdrawals" are positive.

## Type annotations

```python
# `None` if the calculation fails to converge.
# Could return `inf` or `-inf`.
DateLike = Union[str, datetime.date, datetime.datetime, numpy.datetime64, pandas.Timestamp]
Rate = Union[float, Decimal]  # rate as decimal, not percentage, normally between [-1, 1]
Period = Union[int, float, Decimal]
Guess = Optional[Rate]
Amount = Union[int, float, Decimal]
Payment = Tuple[DateLike, Amount]

DateLikeArray = Iterable[DateLike]
AmountArray = Iterable[Amount]
CashFlowSeries = pandas.Series  # with DatetimeIndex
CashFlowTable = Union[Iterable[Payment], pandas.DataFrame, numpy.ndarray]
CashFlowDict = Dict[DateLike, Amount]
CashFlow = Union[CashFlowSeries, CashFlowTable, CashFlowDict]
```

## Day Count Conventions

The [day count convention](https://en.wikipedia.org/wiki/Day_count_convention)
determines how interest accrues over time in a variety of transactions,
including bonds, swaps, bills and loans.

The following conventions are supported:

| Name               | Constant                   | Also known                      |
| ------------------ | -------------------------- | ------------------------------- |
| Actual/Actual ISDA | DayCount.ACT_ACT_ISDA      | Act/Act ISDA                    |
| Actual/365 Fixed   | DayCount.ACT_365F          | Act/365F, English               |
| Actual/365.25      | DayCount.ACT_365_25        |                                 |
| Actual/364         | DayCount.ACT_364           |                                 |
| Actual/360         | DayCount.ACT_360           | French                          |
| 30/360 ISDA        | DayCount.THIRTY_360_ISDA   | Bond basis                      |
| 30E/360            | DayCount.THIRTY_E_360      | 30/360 ISMA, Eurobond basis     |
| 30E+/360           | DayCount.THIRTY_E_PLUS_360 |                                 |
| 30E/360 ISDA       | DayCount.THIRTY_E_360_ISDA | 30E/360 German, German          |
| 30U/360            | DayCount.THIRTY_U_360      | 30/360 US, 30US/360, 30/360 SIA |
| NL/365             | DayCount.NL_365            | Actual/365 No leap year         |
| NL/360             | DayCount.NL_360            |                                 |

Definition:

```python
def year_fraction(
    d1: DateLike,
    d2: DateLike,
    day_count: DayCount,
) -> float:
    ...

```

Usage:

```python
from pyxirr import year_fraction, DayCount
year_fraction("2019-11-09", "2020-03-05", DayCount.THIRTY_E_360)
year_fraction("2019-11-09", "2020-03-05", "act/360")
```

See also:

- [2006 ISDA definitions](https://www.rbccm.com/assets/rbccm/docs/legal/doddfrank/Documents/ISDALibrary/2006%20ISDA%20Definitions.pdf)
- http://www.deltaquants.com/day-count-conventions

## Exceptions

- `InvalidPaymentsError`. Occurs if either:

  - the amounts and dates arrays (`AmountArray`, `DateLikeArray`) are of different lengths
  - the given arrays do not contain at least one negative and at least one positive value

- `BroadcastingError`. Occurs if function arguments could not be broadcast
  together using numpy broadcasting rules.

## numpy-like vectorization

PyXIRR defines a vectorized functions which takes a nested sequence of objects
or numpy arrays as inputs and returns a nested sequence of results of the same shape.

General rules:

- if all input is scalar, returns a scalar float.
- if any input is array_like, returns values for each input element.
- if multiple inputs are array_like, they all must have the same shape (or be
  broadcastable into the same shape).

> PyXIRR has a certain conversion cost compared to numpy-financial. See the charts [here](bench/vectorization/index.html).

> PyXIRR returns a numpy array if the input was a numpy array, otherwise it returns a list.

Examples:

```python
>>> from pyxirr import fv
>>> fv([0.03/12, 0.05/12], 10*12, -100, -100)
[14109.077242352068, 15692.92889433575]
>>> fv(0.05/12, 10*12, [-100, -150], [-100, -50])
[15692.92889433575, 23374.692391734596]
>>> fv([[[0.01]], [[0.02]]], 12, -100, -100)
[[[1380.9328043328946]], [[1468.0331522689821]]]
```

Return numpy array if the input was a numpy array:

```python
>>> import numpy
>>> fv(numpy.array([0.03/12, 0.05/12]), 10*12, -100, -100)
array([14109.07724235, 15692.92889434])
```

## FV

Compute the future value.

```python
def fv(
    rate: Rate,  # Rate of interest per period; scalar or array-like
    nper: Period,  # Number of compounding periods; scalar or array-like
    pmt: Amount,  # Payment; scalar or array-like
    pv: Amount,  # Present value; scalar or array-like
    *,
    pmt_at_beginning: bool = False  # When payments are due; scalar or array-like
) -> Optional[float]:  # returns an array if any input parameter is an array
    ...
```

> Changed in 0.7.0: make pmt_at_beginning keyword-only argument
> Added in 0.9.0: vectorization

The future value is computed by solving the equation:

$$fv + pv \times (1+rate)^{nper}+pmt \times \frac{(1+rate \times when)}{rate} \times ((1+rate)^{nper}-1)=0$$

$$when=\begin{cases}0,&\text{pmt_at_beginning is False}\\1,&\text{pmt_at_beginning is True}\end{cases}$$

in case of `rate == 0`:

$$fv + pv + pmt \times nper = 0$$

#### Examples

What is the future value after 10 years of saving $100 now, with an additional monthly savings of $100. Assume the annual interest rate is 5% compounded monthly?

```python
>>> from pyxirr import fv
>>> fv(0.05/12, 10*12, -100, -100)
15692.92889433575
```

## NFV

Net Future Value

```python
# raises: InvalidPaymentsError (suppressed by passing silent=True flag)
def nfv(
    rate: Rate, # Rate of interest per period
    nper: Period, # Number of compounding periods
    amounts: AmountArray,
    *,
    silent: bool = False
) -> Optional[float]:
    ...
```

Compute the Future Value of uneven payments at regular periods.

The idea is to find the `pv` parameter using the `NPV` function, then calculate `FV` as usual:

```python
import pyxirr

interest_rate = 0.03
payments = [1050, 1350, 1350, 1450]
periods = 6
present_value = pyxirr.npv(interest_rate, payments, start_from_zero=False)
future_value = pyxirr.fv(interest_rate, periods, 0, -present_value)

assert future_value == pyxirr.nfv(interest_rate, periods, payments)
```

See this [video](https://www.youtube.com/watch?v=775ljhriB8U) for details.

## XFV

Future value of a cash flow between two dates.

```python
def xfv(
    start_date: DateLike,
    cash_flow_date: DateLike,
    end_date: DateLike,
    cash_flow_rate: Rate,  # annual rate
    end_rate: Rate,  # annual rate
    cash_flow: Amount,
    *,
    day_count: DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...
```

Where:

- `start_date`: the starting date for the annual interest rates used in the XFV calculation.
- `cash_flow_date`: the date on which the cash flows occurs.
- `end_date`: the ending date for purposes of calculating the future value.
- `cash_flow_rate`: the annual interest rate for the cash flow date. This should be the interest rate from the `start_date` to the `cash_flow_date`.
- `end_rate`: the annual interest rate for the end date. This should be the interest rate from the `start_date` to the `end_date`.
- `cash_flow`: the cash flow value.
- `day_count`: Day count convention.

See also: [XLeratorDB.XFV](http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-XFV-function)

Example:

```python
import pyxirr
from datetime import date

fv = pyxirr.xfv(
    date(2011, 2, 1),
    date(2011, 3, 1),
    date(2012, 2, 1),
    0.00142,
    0.00246,
    100000
)
print(fv)  # 100235.09
```

The result of this calculation means that on 01-Feb-11, we anticipate that
100,000 received on 01-Mar-11 will be worth approximately 100,235.09 on
01-Feb-12, based on the rates provided to the function.

## XNFV

Net Future Value of a series of irregular cash flows.

All cash flows in a group are compounded to the latest cash flow in the group.

```python
# raises: InvalidPaymentsError (suppressed by passing silent=True flag)
def xnfv(
    rate: Rate,  # annual rate
    dates: Union[CashFlow, DateLikeArray],
    amounts: Optional[AmountArray] = None,
    *,
    silent: bool = False
    day_count: DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...
```

See also: [XLeratorDB.XNFV](http://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-XNFV-function)

## PMT

Compute the payment against loan principal plus interest.

```python
def pmt(
    rate: Rate,  # Rate of interest per period; scalar or array-like
    nper: Period,  # Number of compounding periods; scalar or array-like
    pv: Amount,  # Present value; scalar or array-like
    fv: Amount = 0,  # Future value; scalar or array-like
    *,
    pmt_at_beginning: bool = False  # When payments are due; scalar or array-like
) -> Optional[float]:  # returns an array if any input parameter is an array
    ...
```

```
pmt = ipmt + ppmt
```

> Changed in 0.7.0: make pmt_at_beginning keyword-only argument
> Added in 0.9.0: vectorization

See also: [FV](functions.md#fv), [PV](functions.md#pv), [NPER](functions.md#nper)

## IPMT

Compute the interest portion of a payment.

```python
def ipmt(
    rate: Rate,  # Rate of interest per period; scalar or array-like
    per: Period,  # The payment period to calculate the interest amount; scalar or array-like
    nper: Period,  # Number of compounding periods; scalar or array-like
    pv: Amount,  # Present value; scalar or array-like
    fv: Amount = 0,  # Future value; scalar or array-like
    *,
    pmt_at_beginning: bool = False  # When payments are due; scalar or array-like
) -> Optional[float]:  # returns an array if any input parameter is an array
    ...
```

> Changed in 0.7.0: make pmt_at_beginning keyword-only argument
> Added in 0.9.0: vectorization

See also: [PMT](functions.md#pmt)

## PPMT

Compute the payment against loan principal.

```python
def ppmt(
    rate: Rate,  # Rate of interest per period; scalar or array-like
    per: Period,  # The payment period to calculate the interest amount; scalar or array-like
    nper: Period,  # Number of compounding periods; scalar or array-like
    pv: Amount,  # Present value; scalar or array-like
    fv: Amount = 0,  # Future value; scalar or array-like
    *,
    pmt_at_beginning: bool = False  # When payments are due; scalar or array-like
) -> Optional[float]:  # returns an array if any input parameter is an array
    ...
```

> Changed in 0.7.0: make pmt_at_beginning keyword-only argument
> Added in 0.9.0: vectorization

See also: [PMT](functions.md#pmt)

## NPER

Compute the payment against loan principal plus interest.

```python
def nper(
    rate: Rate,  # Rate of interest per period; scalar or array-like
    pmt: Amount,  # Payment; scalar or array-like
    pv: Amount,  # Present value; scalar or array-like
    fv: Amount = 0,  # Future value; scalar or array-like
    *,
    pmt_at_beginning: bool = False  # When payments are due; scalar or array-like
) -> Optional[float]:  # returns an array if any input parameter is an array
    ...
```

> Changed in 0.7.0: make pmt_at_beginning keyword-only argument
> Added in 0.9.0: vectorization

See also: [FV](functions.md#fv), [PV](functions.md#pv), [PMT](functions.md#pmt)

## RATE

Compute the payment against loan principal plus interest.

```python
def rate(
    nper: Period, # Number of compounding periods; scalar or array-like
    pmt: Amount, # Payment; scalar or array-like
    pv: Amount, # Present value; scalar or array-like
    fv: Amount = 0, # Future value; scalar or array-like
    *,
    pmt_at_beginning: bool = False  # When payments are due; scalar or array-like
    guess: Guess = 0.1
) -> Optional[float]:  # returns an array if any input parameter is an array
    ...
```

> Changed in 0.7.0: make pmt_at_beginning and guess keyword-only arguments
> Added in 0.9.0: vectorization

See also: [FV](functions.md#fv), [PV](functions.md#pv), [PMT](functions.md#pmt)

## PV

Compute the present value.

```python
def pv(
    rate: Rate,  # Rate of interest per period; scalar or array-like
    nper: Period,  # Number of compounding periods; scalar or array-like
    pmt: Amount,  # Payment; scalar or array-like
    fv: Amount = 0,  # Future value; scalar or array-like
    *,
    pmt_at_beginning: bool = False  # When payments are due; scalar or array-like
) -> Optional[float]:  # returns an array if any input parameter is an array
    ...
```

> Changed in 0.7.0: make pmt_at_beginning keyword-only argument
> Added in 0.9.0: vectorization

The present value is computed by solving the same equation as for future value:

$$fv+pv \times (1+rate)^{nper}+pmt \times \frac{(1+rate \times when)}{rate} \times ((1+rate)^{nper}-1)=0$$

$$when=\begin{cases}0,&\text{pmt_at_beginning is False}\\1,&\text{pmt_at_beginning is True}\end{cases}$$

in case of `rate == 0`:

$$fv + pv + pmt \times nper = 0$$

#### Examples

What is the present value (e.g., the initial investment) of an investment that needs to total $15692.93 after 10 years of saving $100 every month?
Assume the interest rate is 5% (annually) compounded monthly.

```python
>>> from pyxirr import pv
>>> pv(0.05/12, 10*12, -100, 15692.93)
-100.00067131625819  # so, the initial deposit should be $100
```

## NPV

Compute the Net Present Value.

```python
def npv(
    rate: Rate,
    amounts: AmountArray,
    *,
    start_from_zero=True
) -> Optional[float]:
    ...
```

> Changed in 0.7.0: make start_from_zero keyword-only argument

NPV is calculated using the following formula:

$$\sum_{i=0}^{N-1} \frac{values_i}{(1 + rate)^i}$$

> Values must begin with the initial investment, thus values[0] will typically be negative.
> NPV considers a series of cashflows starting in the present (i = 0). NPV can
> also be defined with a series of future cashflows, paid at the end, rather
> than the start, of each period. If future cashflows are used, the first
> cashflow values[0] must be zeroed and added to the net present value of the
> future cashflows.

> There is a difference between numpy NPV and excel NPV.
> The [numpy docs](https://numpy.org/numpy-financial/latest/npv.html#numpy_financial.npv) show the summation from i=0 to N-1.
> [Excel docs](https://support.microsoft.com/en-us/office/npv-function-8672cb67-2576-4d07-b67b-ac28acf2a568) shows a summation from i=1 to N.
> By default, npv function starts from zero (numpy compatible), but you can call it with `start_from_zero=False` parameter to make it Excel compatible.

#### Examples

```python
>>> from pyxirr import npv
>>> npv(0.08, [-40_000, 5_000, 8_000, 12_000, 30_000])
3065.2226681795255
>>> # Excel compatibility:
>>> npv(0.08, [-40_000, 5_000, 8_000, 12_000, 30_000], start_from_zero=False)
2838.1691372032656
```

It may be preferable to split the projected cashflow into an initial investment
and expected future cashflows. In this case, the value of the initial cashflow
is zero and the initial investment is later added to the future cashflows net
present value.

```python
>>> from pyxirr import npv
>>> npv(0.08, [0, 5_000, 8_000, 12_000, 30_000]) - 40_000
3065.2226681795255
```

## XNPV

Returns the Net Present Value for a schedule of cash flows that is not necessarily periodic.

> To calculate the Net Present Value for a periodic cash flows, use the NPV function.

```python
# raises: InvalidPaymentsError (suppressed by passing silent=True flag)
def xnpv(
    rate: Rate,
    dates: Union[CashFlow, DateLikeArray],
    amounts: Optional[AmountArray] = None,
    *,
    silent: bool = False
    day_count: DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...
```

XNPV is calculated as follows:

$$XNPV=\sum_{i=1}^n \frac{P_i}{(1 + rate)^{(d_i - d_0)/365}}$$

Where:

- `di` = the ith, or last, payment date.
- `d0` = the 0th payment date.
- `Pi` = the ith, or last, payment.

#### Examples

```python
>>> from datetime import date
>>> from pyxirr import xnpv
>>> dates = [date(2020, 1, 1), date(2020, 3, 1), date(2020, 10, 30), date(2021, 2, 15)]
>>> values = [-10_000, 5750, 4250, 3250]
>>> xnpv(0.1, dates, values)
2506.579458169746
```

The function accepts payments in many formats:

- iterable of `tuples` (date, payment)
- `dict` with dates as keys and payments as values
- numpy arrays
- pandas DataFrame and Series

```python
>>> xnpv(0.1, zip(dates, values))
2506.579458169746

>>> xnpv(0.1, dict(zip(dates, values)))
2506.579458169746

>>> import numpy as np
>>> xnpv(0.1, np.array(dates), np.array(values))
2506.579458169746
>>> xnpv(0.1, np.array([dates, values]))
2506.579458169746

>>> import pandas as pd
>>> xnpv(0.1, pd.Series(dates), pd.Series(values))
2506.579458169746
>>> xnpv(0.1, pd.DataFrame(zip(dates, values)))
2506.579458169746
```

The function raises `InvalidPaymentsError` in the following cases:

1. the amounts and dates arrays are of different lengths:

```python
>>> xnpv(0.1, [date(2020, 1, 1)], [-10_000, 5750])
InvalidPaymentsError: the amounts and dates arrays are of different lengths
```

2. values array do not contain at least one negative and at least one positive value:

```python
>>> xnpv(0.1, [date(2020, 1, 1), date(2020, 3, 1)], [-10_000, -5750])
InvalidPaymentsError: negative and positive payments are required
```

## IRR

Compute the Internal Rate of Return.

```python
# raises: InvalidPaymentsError (suppressed by passing silent=True flag)
def irr(
    amounts: AmountArray,
    *,
    guess: Guess = 0.1
    silent: bool = False
) -> Optional[float]:
    ...
```

> Changed in 0.7.0: make guess keyword-only argument

This is the "average" periodically compounded rate of return that gives a [NPV](#npv) of 0.

IRR is the solution of the equation:

$$\sum_{i=0}^n \frac{values_i}{1 + irr}^i = 0$$

#### Examples

```python
>>> from pyxirr import irr, npv
>>> payments = [-100, 39, 59, 55, 20]
>>> irr(payments)
0.2809484212526239

# checking
>>> npv(irr(payments), payments)
0.000000015233
```

The function raises `InvalidPaymentsError` in case of all payments have the same sign:

```python
>>> irr([0, 39, 59, 55, 20])
InvalidPaymentsError: negative and positive payments are required
```

## MIRR

Modified Internal Rate of Return.

```python
# raises: InvalidPaymentsError (suppressed by passing silent=True flag)
def mirr(
    values: AmountArray, # Cash flows. Must contain at least one positive and one negative value or nan is returned.
    finance_rate: Rate, # Interest rate paid on the cash flows
    reinvest_rate: Rate, # Interest rate received on the cash flows upon reinvestment
    *,
    silent: bool = False,
) -> Optional[float]:
    ...
```

MIRR considers both the cost of the investment and the interest received on reinvestment of cash.

The formula for MIRR is

$$MIRR = \left(\frac{-NPV(rrate, values * positive) \times (1 + rrate)^N}{NPV(frate, values * negative) \times (1 + frate)}\right) ^{\frac{1}{N-1}} - 1$$

Where

- `positive` is a unit step function `H(x)`
- `negative` is `1 - H(x)`

Unit step function:

$$H(x):=\begin{cases}1,&{x \gt 0}\\0,&{x \leqslant 0}\end{cases}$$

So the result of:

- `x * positive` => `x * H(x)`
  - 100 _ H(100) = 100 _ 1 = 100
  - -100 _ H(-100) = -100 _ 0 = 100
- `x * negative` => `x * (1 - H(x))`
  - 100 _ (1 - H(100)) = 100 _ (1 - 1) = 0
  - -100 _ (1 - H(-100)) = -100 _ (1 - 0) = -100

## XIRR

Returns the internal rate of return for a schedule of cash flows that is not necessarily periodic.

```python
# raises: InvalidPaymentsError (suppressed by passing silent=True flag)
def xirr(
    dates: Union[CashFlow, DateLikeArray],
    amounts: Optional[AmountArray] = None,
    *,
    guess: Guess = 0.1,
    silent: bool = False,
    day_count: DayCount = DayCount.ACT_365F,
) -> Optional[float]:
    ...
```

> Changed in 0.7.0: make guess keyword-only argument

XIRR is closely related to [XNPV](#xnpv), the Net Present Value function. XIRR is the interest rate corresponding to XNPV = 0.
Library uses an iterative technique for calculating XIRR. If it can't find a result, the `None` value is returned.

XIRR function tries to solve the following equation:

$$\sum_{i=0}^n \frac{P_i}{(1 + rate)^{(d_i - d_0)/365}}=0$$

where:

- `di` = the ith, or last, payment date.
- `d0` = the 0th payment date.
- `Pi` = the ith, or last, payment.

#### Examples

```python
>>> from datetime import date
>>> from pyxirr import xirr
>>> dates = [date(2020, 1, 1), date(2020, 3, 1), date(2020, 10, 30), date(2021, 2, 15)]
>>> values = [-10_000, 5750, 4250, 3250]
>>> xirr(dates, values)
0.6342972615260243

# checking
>>> from pyxirr import xnpv
>>> xnpv(0.6342972615260243, dates, values)
0.0
```

The same input data formats are supported as in the [XNPV](#xnpv) function.

```python
>>> xirr(zip(dates, values))
0.6342972615260243

>>> xirr(dict(zip(dates, values)))
0.6342972615260243

>>> import pandas as pd
>>> xirr(pd.DataFrame(zip(dates, values)))
0.6342972615260243
```

The function raises `InvalidPaymentsError` in the same cases as XNPV.

```python
>>> xirr(dates, values[:-1])
InvalidPaymentsError: the amounts and dates arrays are of different lengths
>>> xirr(dates, [abs(x) for x in values])
InvalidPaymentsError: negative and positive payments are required
```
