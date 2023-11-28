Use the Long-Nickels method to re-calculate the private equity nav to match the
public market equivalents (PME) for comparison. This method just re-calculates
the nav. Instead of relying on the given nav, it is calculated as the future
valued contributions less the future valued distributions.

This will look like (for two periods with a contribution and distribution in each):
```
nav = c[1] * index[-1]/index[1] + c[2] * index[-1]/index[2]
        - d[1] * index[-1]/index[1] - d[2] * index[-1]/index[2]
```

See also:
- <https://en.wikipedia.org/wiki/Public_Market_Equivalent#Long-Nickels_PME>
- <https://blog.edda.co/advanced-fund-performance-methods-pme-direct-alpha/>
