use libc::{c_double, c_long, c_ushort, size_t};

use pyxirr_core::{self as core, DateLike, DayCount};

fn day_count_from_code(value: u16) -> Option<DayCount> {
    match value {
        0 => Some(DayCount::ACT_365F),
        1 => Some(DayCount::ACT_ACT_ISDA),
        2 => Some(DayCount::ACT_365_25),
        3 => Some(DayCount::ACT_364),
        4 => Some(DayCount::ACT_360),
        5 => Some(DayCount::THIRTY_360_ISDA),
        6 => Some(DayCount::THIRTY_E_360),
        7 => Some(DayCount::THIRTY_E_PLUS_360),
        8 => Some(DayCount::THIRTY_E_360_ISDA),
        9 => Some(DayCount::THIRTY_U_360),
        10 => Some(DayCount::NL_365),
        11 => Some(DayCount::NL_360),
        _ => None,
    }
}

#[repr(i32)]
pub enum ReturnCode {
    Success = 0,
    NullReference = 1,
    InvalidDayCount = 2,
    ArraysOfDifferentLength = 3,
    PositiveAndNegativePaymentsRequired = 4,
}

#[no_mangle]
pub unsafe extern "C" fn xnpv(
    rate: c_double,
    dates_ptr: *const c_long,
    dates_length: size_t,
    values_ptr: *const c_double,
    values_length: size_t,
    day_count: c_ushort,
    result: *mut c_double,
) -> ReturnCode {
    if values_ptr.is_null() || dates_ptr.is_null() {
        return ReturnCode::NullReference;
    }

    if values_length != dates_length {
        return ReturnCode::ArraysOfDifferentLength;
    }

    let day_count = match day_count_from_code(day_count) {
        Some(x) => x,
        None => return ReturnCode::InvalidDayCount,
    };

    let values = std::slice::from_raw_parts(values_ptr, values_length);
    let timestamps = std::slice::from_raw_parts(dates_ptr, dates_length);

    let dates: Vec<_> = timestamps.iter().map(|ts| DateLike::from_unix_timestamp(*ts)).collect();

    match core::xnpv(rate, &dates, values, Some(day_count)) {
        Ok(value) => {
            *result = value;
            ReturnCode::Success
        }
        // because the length of the array has already been checked
        Err(_) => ReturnCode::PositiveAndNegativePaymentsRequired,
    }
}

#[no_mangle]
pub unsafe extern "C" fn xirr(
    dates_ptr: *const c_long,
    dates_length: size_t,
    values_ptr: *const c_double,
    values_length: size_t,
    guess: c_double,
    day_count: c_ushort,
    result: *mut c_double,
) -> ReturnCode {
    if values_ptr.is_null() || dates_ptr.is_null() {
        return ReturnCode::NullReference;
    }

    if values_length != dates_length {
        return ReturnCode::ArraysOfDifferentLength;
    }

    let day_count = match day_count_from_code(day_count) {
        Some(x) => x,
        None => return ReturnCode::InvalidDayCount,
    };

    let values = std::slice::from_raw_parts(values_ptr, values_length);
    let timestamps = std::slice::from_raw_parts(dates_ptr, dates_length);

    let dates: Vec<_> = timestamps.iter().map(|ts| DateLike::from_unix_timestamp(*ts)).collect();

    match core::xirr(&dates, values, Some(guess), Some(day_count)) {
        Ok(rate) => {
            *result = rate;
            ReturnCode::Success
        }
        // because the length of the array has already been checked
        Err(_) => ReturnCode::PositiveAndNegativePaymentsRequired,
    }
}
