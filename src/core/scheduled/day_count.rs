use std::{cmp::min, fmt, str::FromStr};

use time::{
    util::{days_in_year_month, is_leap_year},
    Date, Month,
};

#[pyo3::pyclass]
#[pyo3(frozen)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
#[derive(Default)]
pub enum DayCount {
    ACT_ACT_ISDA,
    #[default]
    ACT_365F,
    ACT_365_25,
    ACT_364,
    ACT_360,
    THIRTY_360_ISDA,
    THIRTY_E_360,
    THIRTY_E_PLUS_360,
    THIRTY_E_360_ISDA,
    THIRTY_U_360,
    NL_365,
    NL_360,
}

impl fmt::Display for DayCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DayCount::*;
        let repr = match self {
            ACT_ACT_ISDA => "Actual/Actual ISDA",
            ACT_365F => "Actual/365F",
            ACT_365_25 => "Actual/365.25",
            ACT_364 => "Actual/364",
            ACT_360 => "Actual/360",
            THIRTY_360_ISDA => "30/360 ISDA",
            THIRTY_E_360 => "30E/360",
            THIRTY_E_PLUS_360 => "30E+/360",
            THIRTY_E_360_ISDA => "30E/360 ISDA",
            THIRTY_U_360 => "30U/360",
            NL_365 => "NL/365",
            NL_360 => "NL/360",
        };
        write!(f, "{}", repr)
    }
}

impl FromStr for DayCount {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "actual/actual" | "actual/actual isda" | "act/act" | "act/act isda" => {
                Ok(DayCount::ACT_ACT_ISDA)
            }
            #[rustfmt::skip]
            "actual/365 fixed" | "act/365 fixed" | "a/365 fixed"
                | "actual/365f" | "act/365f" | "a/365f"
                | "english" => Ok(DayCount::ACT_365F),
            "actual/365.25" | "act/365.25" | "a/365.25" => Ok(DayCount::ACT_365_25),
            "actual/364" | "act/364" | "a/364" => Ok(DayCount::ACT_364),
            "actual/360" | "act/360" | "a/360" | "french" => Ok(DayCount::ACT_360),
            "30/360" | "30/360 isda" | "bond basis" => Ok(DayCount::THIRTY_360_ISDA),
            "30e/360" | "30/360 isma" | "eurobond basis" => Ok(DayCount::THIRTY_E_360),
            "30u/360" | "30us/360" | "30/360 us" | "30/360 sia" => Ok(DayCount::THIRTY_U_360),
            "30e+/360" => Ok(DayCount::THIRTY_E_PLUS_360),
            "30e/360 isda" | "30e/360 german" | "german" => Ok(DayCount::THIRTY_E_360_ISDA),
            "nl/365" | "nl365" => Ok(DayCount::NL_365),
            "nl/360" | "nl360" => Ok(DayCount::NL_360),
            _ => Err("Invalid Day Count Convention"),
        }
    }
}

pub fn year_fraction<T: AsRef<Date>>(d1: T, d2: T, day_count: DayCount) -> f64 {
    let (d1, d2) = (d1.as_ref(), d2.as_ref());

    use DayCount::*;
    match day_count {
        ACT_ACT_ISDA => {
            let (normal_days, leap_days) = normal_leap_days(d1, d2);
            normal_days as f64 / 365.0 + leap_days as f64 / 366.0
        }
        ACT_365F => days_between_act(d1, d2) as f64 / 365.0,
        ACT_365_25 => days_between_act(d1, d2) as f64 / 365.25,
        ACT_364 => days_between_act(d1, d2) as f64 / 364.0,
        ACT_360 => days_between_act(d1, d2) as f64 / 360.0,
        THIRTY_360_ISDA => days_between_30_360_isda(d1, d2) as f64 / 360.0,
        THIRTY_E_360 => days_between_30_e_360(d1, d2) as f64 / 360.0,
        THIRTY_E_PLUS_360 => days_between_30_e_plus_360(d1, d2) as f64 / 360.0,
        THIRTY_E_360_ISDA => days_between_30_e_360_isda(d1, d2) as f64 / 360.0,
        THIRTY_U_360 => days_between_30_u_360(d1, d2) as f64 / 360.0,
        NL_365 => days_between_excluding_leap_days(d1, d2) as f64 / 365.0,
        NL_360 => days_between_excluding_leap_days(d1, d2) as f64 / 360.0,
    }
}

pub fn days_between<T: AsRef<Date>>(d1: T, d2: T, day_count: DayCount) -> i32 {
    let (d1, d2) = (d1.as_ref(), d2.as_ref());

    use DayCount::*;
    match day_count {
        ACT_ACT_ISDA | ACT_365F | ACT_365_25 | ACT_364 | ACT_360 => days_between_act(d1, d2),
        THIRTY_360_ISDA => days_between_30_360_isda(d1, d2),
        THIRTY_E_360 => days_between_30_e_360(d1, d2),
        THIRTY_E_PLUS_360 => days_between_30_e_plus_360(d1, d2),
        THIRTY_E_360_ISDA => days_between_30_e_360_isda(d1, d2),
        THIRTY_U_360 => days_between_30_u_360(d1, d2),
        NL_365 => days_between_excluding_leap_days(d1, d2),
        NL_360 => days_between_excluding_leap_days(d1, d2),
    }
}

fn days_between_act(d1: &Date, d2: &Date) -> i32 {
    d2.to_julian_day() - d1.to_julian_day()
}

fn normal_leap_years(d1: &Date, d2: &Date) -> (i32, i32) {
    let (y1, y2) = (d1.year(), d2.year());
    let leap_years = (y1..=y2).filter(|&y| is_leap_year(y)).count() as i32;
    let normal_years = y2 - y1 - leap_years + 1;
    (normal_years, leap_years)
}

fn normal_leap_days(d1: &Date, d2: &Date) -> (i32, i32) {
    let (mut normal, mut leap) = normal_leap_years(d1, d2);

    normal *= 365;
    leap *= 366;

    if is_leap_year(d1.year()) {
        leap -= d1.ordinal() as i32 - 1
    } else {
        normal -= d1.ordinal() as i32 - 1
    }

    if is_leap_year(d2.year()) {
        leap -= 366 - d2.ordinal() as i32 + 1
    } else {
        normal -= 365 - d2.ordinal() as i32 + 1
    }

    (normal, leap)
}

fn days_between_excluding_leap_days(d1: &Date, d2: &Date) -> i32 {
    days_between_act(d1, d2) - leap_days_between(d1, d2)
}

fn leap_days_between(d1: &Date, d2: &Date) -> i32 {
    let (_, mut leap_days) = normal_leap_years(d1, d2);

    if is_leap_year(d1.year()) && (d1.month() as u8) > 2 {
        leap_days -= 1
    }

    if is_leap_year(d2.year()) && d2.ordinal() < 60 {
        leap_days -= 1
    }

    leap_days
}

fn days_between_30_360_isda(d1: &Date, d2: &Date) -> i32 {
    let d1_day = min(d1.day(), 30);
    let d2_day = if d1_day >= 30 {
        min(d2.day(), 30)
    } else {
        d2.day()
    };
    days_between_30_360(d1, d2, d1_day, d2_day)
}

fn days_between_30_e_360(d1: &Date, d2: &Date) -> i32 {
    let d1_day = min(d1.day(), 30);
    let d2_day = min(d2.day(), 30);
    days_between_30_360(d1, d2, d1_day, d2_day)
}

fn days_between_30_e_360_isda(d1: &Date, d2: &Date) -> i32 {
    let d1_day = if is_last_day_of_feb(d1) {
        30
    } else {
        min(d1.day(), 30)
    };
    let d2_day = if is_last_day_of_feb(d2) {
        30
    } else {
        min(d2.day(), 30)
    };
    days_between_30_360(d1, d2, d1_day, d2_day)
}

fn days_between_30_e_plus_360(d1: &Date, d2: &Date) -> i32 {
    let d1_day = min(d1.day(), 30);
    if d2.day() == 31 {
        let d2 = d2.next_day().unwrap();
        days_between_30_360(d1, &d2, d1_day, d2.day())
    } else {
        days_between_30_360(d1, d2, d1_day, d2.day())
    }
}

fn days_between_30_u_360(d1: &Date, d2: &Date) -> i32 {
    let d1_day = if is_last_day_of_feb(d1) {
        30
    } else {
        min(d1.day(), 30)
    };
    let d2_day = if is_last_day_of_feb(d1) && is_last_day_of_feb(d2) {
        30
    } else if d1_day >= 30 {
        min(d2.day(), 30)
    } else {
        d2.day()
    };

    days_between_30_360(d1, d2, d1_day, d2_day)
}

fn days_between_30_360(d1: &Date, d2: &Date, d1_day: u8, d2_day: u8) -> i32 {
    360 * (d2.year() - d1.year())
        + 30 * (d2.month() as i32 - d1.month() as i32)
        + (d2_day as i32 - d1_day as i32)
}

pub fn is_last_day_of_month(date: &Date) -> bool {
    date.day() == days_in_year_month(date.year(), date.month())
}

pub fn is_last_day_of_feb(date: &Date) -> bool {
    date.month() == Month::February && is_last_day_of_month(date)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::core::DateLike;

    // test cases from http://www.deltaquants.com/day-count-conventions
    #[rstest]
    #[case("Act/Act", 4./365. + 58./366.)]
    #[case("Act/365F", 62./365.)]
    #[case("Act/360", 62./360.)]
    // #[case("Act/365A", 62./365.)]
    // #[case("Act/365L", 62./366.)]
    #[case("NL/365", 62./365.)]
    #[case("30/360 ISDA", 60./360.)]
    #[case("30E/360", 60./360.)]
    #[case("30E+/360", 60./360.)]
    #[case("30E/360 German", 60./360.0)]
    #[case("30/360 US", 60./360.)]
    fn test_example_1(#[case] day_count: DayCount, #[case] expected: f64) {
        let d1 = "2007-12-28".parse::<DateLike>().unwrap();
        let d2 = "2008-02-28".parse::<DateLike>().unwrap();
        assert_eq!(year_fraction(&d1, &d2, day_count), expected);
    }

    #[rstest]
    #[case("Act/Act", 4./365. + 59./366.)]
    #[case("Act/365F", 63./365.)]
    #[case("Act/360", 63./360.)]
    // #[case("Act/365A", 63./366.)]
    // #[case("Act/365L", 63./366.)]
    #[case("NL/365", 62./365.)]
    #[case("30/360 ISDA", 61./360.)]
    #[case("30E/360", 61./360.)]
    #[case("30E+/360", 61./360.)]
    #[case("30E/360 German", 62./360.)]
    #[case("30/360 US", 61./360.)]
    fn test_example_2(#[case] day_count: DayCount, #[case] expected: f64) {
        let d1 = "2007-12-28".parse::<DateLike>().unwrap();
        let d2 = "2008-02-29".parse::<DateLike>().unwrap();
        assert_eq!(year_fraction(&d1, &d2, day_count), expected);
    }

    #[rstest]
    #[case("Act/Act", 62./365. + 334./366.)]
    #[case("Act/365F", 396./365.)]
    #[case("Act/360", 396./360.)]
    // #[case("Act/365A", 396./366.)]
    // #[case("Act/365L", 396./366.)]
    #[case("NL/365", 395./365.)]
    #[case("30/360 ISDA", 390./360.)]
    #[case("30E/360", 390./360.)]
    #[case("30E+/360", 390./360.)]
    #[case("30E/360 German", 390./360.)]
    #[case("30/360 US", 390./360.)]
    fn test_example_3(#[case] day_count: DayCount, #[case] expected: f64) {
        let d1 = "2007-10-31".parse::<DateLike>().unwrap();
        let d2 = "2008-11-30".parse::<DateLike>().unwrap();
        assert_eq!(year_fraction(&d1, &d2, day_count), expected);
    }

    #[rstest]
    #[case("Act/Act", 335./366. + 150./365.)]
    #[case("Act/365F", 485./365.)]
    #[case("Act/360", 485./360.)]
    // #[case("Act/365A", 485./366.)]
    // #[case("Act/365L", 485./365.)]
    #[case("NL/365", 484./365.)]
    #[case("30/360 ISDA", 480./360.)]
    #[case("30E/360", 479./360.)]
    #[case("30E+/360", 480./360.)]
    #[case("30E/360 German", 479./360.)]
    #[case("30/360 US", 480./360.)]
    fn test_example_4(#[case] day_count: DayCount, #[case] expected: f64) {
        let d1 = "2008-02-01".parse::<DateLike>().unwrap();
        let d2 = "2009-05-31".parse::<DateLike>().unwrap();
        assert_eq!(year_fraction(&d1, &d2, day_count), expected);
    }

    // https://westclintech.com/SQL-Server-Financial-Functions/SQL-Server-DAYS360-function
    // Start Date, End Date, 30/360 US, 30E/360, 30E360 ISDA, Actual
    static XLERATOR_DB_DATA: [(&str, &str, i32, i32, i32, i32); 22] = [
        ("2007-01-15", "2007-01-30", 15, 15, 15, 15),
        ("2007-01-15", "2007-02-15", 30, 30, 30, 31),
        ("2007-01-15", "2007-07-15", 180, 180, 180, 181),
        ("2007-09-30", "2008-03-31", 180, 180, 180, 183),
        ("2007-09-30", "2007-10-31", 30, 30, 30, 31),
        ("2007-09-30", "2008-09-30", 360, 360, 360, 366),
        ("2007-01-15", "2007-01-31", 16, 15, 15, 16),
        ("2007-01-31", "2007-02-28", 28, 28, 30, 28),
        ("2007-02-28", "2007-03-31", 30, 32, 30, 31),
        ("2006-08-31", "2007-02-28", 178, 178, 180, 181),
        ("2007-02-28", "2007-08-31", 180, 182, 180, 184),
        ("2007-02-14", "2007-02-28", 14, 14, 16, 14),
        ("2007-02-26", "2008-02-29", 363, 363, 364, 368),
        ("2008-02-29", "2009-02-28", 360, 359, 360, 365),
        ("2008-02-29", "2008-03-30", 30, 31, 30, 30),
        ("2008-02-29", "2008-03-31", 30, 31, 30, 31),
        ("2007-02-28", "2007-03-05", 5, 7, 5, 5),
        ("2007-10-31", "2007-11-28", 28, 28, 28, 28),
        ("2007-08-31", "2008-02-29", 179, 179, 180, 182),
        ("2008-02-29", "2008-08-31", 180, 181, 180, 184),
        ("2008-08-31", "2009-02-28", 178, 178, 180, 181),
        ("2009-02-28", "2009-08-31", 180, 182, 180, 184),
    ];

    #[rstest]
    fn test_from_xleratordb() {
        for row in XLERATOR_DB_DATA {
            let d1 = &row.0.parse::<DateLike>().unwrap();
            let d2 = &row.1.parse::<DateLike>().unwrap();
            assert_eq!(days_between(d1, d2, DayCount::THIRTY_U_360), row.2);
            assert_eq!(days_between(d1, d2, DayCount::THIRTY_E_360), row.3);
            assert_eq!(days_between(d1, d2, DayCount::ACT_365F), row.5);
        }
    }
}
