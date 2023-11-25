Use the PME+ method to re-scale the private equity flows to match the public
market equivalents (PME) for comparison. This method works as follows: create
an equation that sets the nav equal to the contributions future valued based on
the PME returns, minus the distributions multiplied by a scalar (lambda) future
valued based on the PME returns.

This will look like (for two periods with a contribution and distribution in each):

nav = c1 * px_final/px_1 + c2 * px_final/px_2
        - d1 * lambda * px_final/px_1 - d2 * lambda * px_final/px_2

Solve for lambda so that the two sides of the equation are equal. Then multiply
all the distributions by lambda to re-scale them.
