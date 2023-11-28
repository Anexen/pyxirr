Use the PME+ method to re-scale the private equity flows to match the public
market equivalents (PME) for comparison. This method works as follows: create
an equation that sets the NAV equal to the contributions future valued based on
the PME returns, minus the distributions multiplied by a scalar (lambda) future
valued based on the PME returns.

This will look like (for two periods with a contribution and distribution in each):
```
nav = c[1] * index[-1]/index[1] + c[2] * index[-1]/index[2]
        - d[1] * λ * index[-1]/index[1] - d[2] * λ * index[-1]/index[2]
```
Solve for lambda so that the two sides of the equation are equal. Then multiply
all the distributions by lambda to re-scale them.

See also:
- <https://en.wikipedia.org/wiki/Public_Market_Equivalent#PME+>
- <https://blog.edda.co/advanced-fund-performance-methods-pme-direct-alpha/>
