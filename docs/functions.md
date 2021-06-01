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

$$when=\begin{cases}0,&\text{pmt_at_begining is True}\\1,&\text{pmt_at_begining is False}\end{cases}$$

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
