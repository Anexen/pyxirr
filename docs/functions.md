{% include head.html %}

# Functions

## Type annotations

```python
# `None` if the calculation fails to converge or result is NaN.
# could return `inf` or `-inf`
FloatOrNone = Optional[float]

DateLike = Union[datetime.date, datetime.datetime, numpy.datetime64, pandas.Timestamp]
Rate = float  # rate as decimal, not percentage, normally between [-1, 1]
Guess = Optional[Rate]
Amount = Union[int, float, Decimal]
Payment = Tuple[DateLike, Amount]

DateLikeArray = Iterable[DateLike]
AmountArray = Iterable[Amount]
CashFlowTable = Iterable[Payment]
CashFlowDict = Dict[DateLike, Amount]
```

## Exceptions

- `InvalidPaymentsError`. Occurs if either:
  - the amounts and dates arrays (`AmountArray`, `DateLikeArray`) are of different lengths
  - the given arrays do not contain at least one negative and at least one positive value

## FV

Compute the future value.

```python
fv(
    rate: Rate, # Rate of interest per period
    nper: int, # Number of compounding periods
    pmt: Amount, # Payment
    pv: Amount, # Present value
    pmt_at_begining: bool = False  # When payments are due
) -> FloatOrNone
```

The future value is computed by solving the equation:

$$fv+pv*(1+rate)^{nper}+pmt*\frac{(1+rate*when)}{rate}*((1+rate)^{nper}-1)=0$$

$$when=\begin{cases}0,&\text{pmt_at_begining is False}\\1,&\text{pmt_at_begining is True}\end{cases}$$

in case of `rate` == 0:

$$fv + pv + pmt * nper = 0$$

#### Examples

What is the future value after 10 years of saving $100 now, with an additional monthly savings of $100. Assume the annual interest rate is 5% compounded monthly?
> By convention, the negative sign represents cash flow out (i.e. money not available today).

```python
>>> from pyxirr import fv
>>> fv(0.05/12, 10*12, -100, -100)
15692.92889433575
```

## PV

Compute the present value.

```python
pv(
    rate: Rate, # Rate of interest per period
    nper: int, # Number of compounding periods
    pmt: Amount, # Payment
    fv: Amount = 0, # Future value
    pmt_at_begining: bool = False  # When payments are due
) -> FloatOrNone
```

## NPV

Compute the Net Present Value.

```python
# raises: InvalidPaymentsError
npv(rate: Rate, amounts: AmountArray) -> FloatOrNone
```

## XNPV

```python
# raises: InvalidPaymentsError
xnpv(
    rate: Rate,
    dates: Union[CashFlowTable, CashFlowDict, DateLikeArray],
    amounts: Optional[AmountArray] = None,
) -> FloatOrNone
```

## IRR

Compute the Internal Rate of Return (IRR)

```python
# raises: InvalidPaymentsError
irr(amounts: AmountArray, guess: Guess = 0.1) -> FloatOrNone
```

## MIRR

Modified internal rate of return.

```python
mirr(
    values: AmountArray, # Cash flows. Must contain at least one positive and one negative value or nan is returned.
    finance_rate: Rate, # Interest rate paid on the cash flows
    reinvest_rate: Rate, # Interest rate received on the cash flows upon reinvestment
) -> FloatOrNone
```

## XIRR

Returns the internal rate of return for a schedule of cash flows that is not necessarily periodic.

```python
# raises: InvalidPaymentsError
xirr(
    dates: Union[CashFlowTable, CashFlowDict, DateLikeArray],
    amounts: Optional[AmountArray] = None,
    guess: Guess = 0.1,
) -> FloatOrNone
```

XIRR is closely related to XNPV, the net present value function. XIRR is the interest rate corresponding to XNPV = 0.
Library uses an iterative technique for calculating XIRR. If it can't find a result, the `None` value is returned.

XIRR function tries to solve the following equation:

$$\displaystyle\sum_{i=1}^n \frac{P_i}{(1 + rate)^{(d_i - d_0)/365}} = 0$$

where:

- `di` = the ith, or last, payment date.
- `d0` = the 0th payment date.
- `Pi` = the ith, or last, payment.
