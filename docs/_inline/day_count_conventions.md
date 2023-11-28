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

See also:

- [2006 ISDA definitions](https://www.rbccm.com/assets/rbccm/docs/legal/doddfrank/Documents/ISDALibrary/2006%20ISDA%20Definitions.pdf)
- http://www.deltaquants.com/day-count-conventions
