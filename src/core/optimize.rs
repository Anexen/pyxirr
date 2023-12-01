const MAX_ERROR: f64 = 1e-9;
const MAX_ITERATIONS: u32 = 50;

pub fn newton_raphson<Func, Deriv>(start: f64, f: &Func, d: &Deriv) -> f64
where
    Func: Fn(f64) -> f64,
    Deriv: Fn(f64) -> f64,
{
    // x[n + 1] = x[n] - f(x[n])/f'(x[n])

    let mut x = start;

    for _ in 0..MAX_ITERATIONS {
        let res = f(x);

        if res.abs() < MAX_ERROR {
            return x;
        }

        let delta = res / d(x);

        if delta.abs() < MAX_ERROR {
            return x - delta;
        }

        x -= delta;
    }

    f64::NAN
}

pub fn newton_raphson_with_default_deriv<Func>(start: f64, f: Func) -> f64
where
    Func: Fn(f64) -> f64,
{
    // deriv = (f(x + e) - f(x - e))/((x + e) - x)
    // multiply denominator by 2 for faster convergence

    // https://programmingpraxis.com/2012/01/13/excels-xirr-function/

    let df = |x| (f(x + MAX_ERROR) - f(x - MAX_ERROR)) / (2.0 * MAX_ERROR);
    newton_raphson(start, &f, &df)
}

// https://github.com/scipy/scipy/blob/39bf11b96f771dcecf332977fb2c7843a9fd55f2/scipy/optimize/Zeros/brentq.c
pub fn brentq<Func>(f: &Func, xa: f64, xb: f64, iter: usize) -> f64
where
    Func: Fn(f64) -> f64,
{
    let xtol = 2e-14;
    let rtol = 8.881784197001252e-16;

    let mut xpre = xa;
    let mut xcur = xb;
    let (mut xblk, mut fblk, mut spre, mut scur) = (0., 0., 0., 0.);
    /* the tolerance is 2*delta */

    let mut fpre = f(xpre);
    let mut fcur = f(xcur);

    if fpre.signum() == fcur.signum() {
        return f64::NAN; // sign error
    }
    if fpre == 0. {
        return xpre;
    }
    if fcur == 0. {
        return xcur;
    }

    for _ in 0..iter {
        if fpre != 0. && fcur != 0. && fpre.signum() != fcur.signum() {
            xblk = xpre;
            fblk = fpre;
            spre = xcur - xpre;
            scur = spre;
        }

        if fblk.abs() < fcur.abs() {
            xpre = xcur;
            xcur = xblk;
            xblk = xpre;

            fpre = fcur;
            fcur = fblk;
            fblk = fpre;
        }

        let delta = (xtol + rtol * xcur.abs()) / 2.;
        let sbis = (xblk - xcur) / 2.;

        if fcur == 0. || sbis.abs() < delta {
            return xcur;
        }

        if spre.abs() > delta && fcur.abs() < fpre.abs() {
            let stry = if xpre == xblk {
                /* interpolate */
                -fcur * (xcur - xpre) / (fcur - fpre)
            } else {
                /* extrapolate */
                let dpre = (fpre - fcur) / (xpre - xcur);
                let dblk = (fblk - fcur) / (xblk - xcur);
                -fcur * (fblk * dblk - fpre * dpre) / (dblk * dpre * (fblk - fpre))
            };

            if 2. * stry.abs() < spre.abs().min(3. * sbis.abs() - delta) {
                /* good short step */
                spre = scur;
                scur = stry;
            } else {
                /* bisect */
                spre = sbis;
                scur = sbis;
            }
        } else {
            /* bisect */
            spre = sbis;
            scur = sbis;
        }

        xpre = xcur;
        fpre = fcur;
        if scur.abs() > delta {
            xcur += scur;
        } else {
            xcur += if sbis > 0. {
                delta
            } else {
                -delta
            }
        }

        fcur = f(xcur);
    }

    f64::NAN
}

pub fn brentq_grid_search<'a, Func>(
    breakpoints: &'a [&[f64]],
    f: &'a Func,
) -> impl Iterator<Item = f64> + 'a
where
    Func: Fn(f64) -> f64 + 'a,
{
    breakpoints
        .into_iter()
        .flat_map(|x| x.windows(2).map(|pair| brentq(f, pair[0], pair[1], 100)))
        .filter(|r| r.is_finite() && f(*r).abs() < 1e-3)
}
