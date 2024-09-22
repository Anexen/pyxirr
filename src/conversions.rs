use time::Date;

use crate::core::DateLike;

// time::Date::from_ordinal_date(1970, 1).unwrap().to_julian_day();
static UNIX_EPOCH_JULIAN_DAY: i32 = 2440588;

struct DaysSinceUnixEpoch(i32);

impl From<DaysSinceUnixEpoch> for DateLike {
    fn from(value: DaysSinceUnixEpoch) -> Self {
        Date::from_julian_day(UNIX_EPOCH_JULIAN_DAY + value.0).unwrap().into()
    }
}

impl From<i64> for DateLike {
    fn from(value: i64) -> Self {
        Date::from_julian_day(UNIX_EPOCH_JULIAN_DAY + (value as i32)).unwrap().into()
    }
}

pub struct AmountArray(Vec<f64>);

impl std::ops::Deref for AmountArray {
    type Target = [f64];

    fn deref(&self) -> &[f64] {
        self.0.as_ref()
    }
}
