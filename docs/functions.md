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

## FV
## PV
## NPV
## XNPV
## IRR
## MIRR

## XIRR

Returns the internal rate of return for a schedule of cash flows that is not necessarily periodic.

```
# raises: InvalidPaymentsError

pyxirr.xirr(
    dates: Union[CashFlowTable, CashFlowDict, DateLikeArray],
    amounts: Optional[AmountArray] = None,
    guess: Guess = 0.1,
) -> FloatOrNone
```

XIRR is closely related to XNPV, the net present value function. XIRR is the interest rate corresponding to XNPV = 0.
Library uses an iterative technique for calculating XIRR. If it can't find a result, the None value is returned.

XIRR function tries to solve the following equation:

<img src="https://render.githubusercontent.com/render/math?math=\sum_{i=1}^n \frac{P_i}{(1 + rate)^{(d_i - d_0)/365}} = 0">

where:
- `di` = the ith, or last, payment date.
- `d0` = the 0th payment date.
- `Pi` = the ith, or last, payment.
