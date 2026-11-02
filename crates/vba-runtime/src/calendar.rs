use crate::{Error, Result};

const OA_EPOCH_UNIX_DAYS: i64 = -25_569;
const SECONDS_PER_DAY: f64 = 86_400.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DateParts {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub seconds: u32,
    pub subsecond_micros: u32,
}

fn invalid_date() -> Error {
    Error::Vba {
        number: 5,
        message: "Invalid procedure call or argument".into(),
    }
}
fn overflow() -> Error {
    Error::Vba {
        number: 6,
        message: "Overflow".into(),
    }
}
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let adjusted_month = i64::from(month) + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}
fn civil_from_days(days: i64) -> (i32, u8, u8) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year as i32, month as u8, day as u8)
}
pub(crate) fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
pub(crate) fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}
pub(crate) fn serial_from_ymd(year: i32, month: u8, day: u8) -> Result<f64> {
    if !(100..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
    {
        return Err(invalid_date());
    }
    Ok((days_from_civil(year, u32::from(month), u32::from(day)) - OA_EPOCH_UNIX_DAYS) as f64)
}
pub(crate) fn date_serial(year: i64, month: i64, day: i64) -> Result<f64> {
    if !(-32_768..=32_767).contains(&year)
        || !(-32_768..=32_767).contains(&month)
        || !(-32_768..=32_767).contains(&day)
    {
        return Err(overflow());
    }
    let year = match year {
        0..=29 => year + 2000,
        30..=99 => year + 1900,
        _ => year,
    };
    let month_zero = month.checked_sub(1).ok_or_else(overflow)?;
    let year = year
        .checked_add(month_zero.div_euclid(12))
        .ok_or_else(overflow)?;
    let month = month_zero.rem_euclid(12) + 1;
    if !(100..=9999).contains(&year) {
        return Err(invalid_date());
    }
    let base = days_from_civil(year as i32, month as u32, 1);
    let days = base.checked_add(day - 1).ok_or_else(overflow)?;
    let (year, month, day) = civil_from_days(days);
    serial_from_ymd(year, month, day)
}
pub(crate) fn from_serial(serial: f64) -> Result<DateParts> {
    if !serial.is_finite() {
        return Err(overflow());
    }
    let day = serial.trunc();
    if day < i64::MIN as f64 || day > i64::MAX as f64 {
        return Err(overflow());
    }
    let (year, month, date) = civil_from_days(day as i64 + OA_EPOCH_UNIX_DAYS);
    if !(100..=9999).contains(&year) {
        return Err(invalid_date());
    }
    let fraction = serial.fract().abs();
    let micros = (fraction * SECONDS_PER_DAY * 1_000_000.0).round();
    if !(0.0..=SECONDS_PER_DAY * 1_000_000.0).contains(&micros) {
        return Err(overflow());
    }
    let micros = if micros >= SECONDS_PER_DAY * 1_000_000.0 {
        0
    } else {
        micros as u64
    };
    Ok(DateParts {
        year,
        month,
        day: date,
        seconds: (micros / 1_000_000) as u32,
        subsecond_micros: (micros % 1_000_000) as u32,
    })
}
pub(crate) fn to_serial(parts: DateParts) -> Result<f64> {
    let date = serial_from_ymd(parts.year, parts.month, parts.day)?;
    let fraction = (f64::from(parts.seconds) + f64::from(parts.subsecond_micros) / 1_000_000.0)
        / SECONDS_PER_DAY;
    let result = if date < 0.0 {
        date - fraction
    } else {
        date + fraction
    };
    from_serial(result)?;
    Ok(result)
}
pub(crate) fn weekday_sunday_one(serial_day: i64) -> u8 {
    (serial_day + 6).rem_euclid(7) as u8 + 1
}
pub(crate) fn serial_day(serial: f64) -> Result<i64> {
    from_serial(serial)?;
    Ok(serial.trunc() as i64)
}
pub(crate) fn civil_from_serial_day(days: i64) -> Result<(i32, u8, u8)> {
    let (year, month, day) = civil_from_days(days + OA_EPOCH_UNIX_DAYS);
    if !(100..=9999).contains(&year) {
        return Err(invalid_date());
    }
    Ok((year, month, day))
}
pub(crate) fn add_months(year: i32, month: u8, day: u8, amount: i64) -> Result<(i32, u8, u8)> {
    let month_index = i64::from(year)
        .checked_mul(12)
        .and_then(|n| n.checked_add(i64::from(month) - 1))
        .and_then(|n| n.checked_add(amount))
        .ok_or_else(overflow)?;
    let target_year = month_index.div_euclid(12);
    if !(100..=9999).contains(&target_year) {
        return Err(invalid_date());
    }
    let target_month = (month_index.rem_euclid(12) + 1) as u8;
    let target_day = day.min(days_in_month(target_year as i32, target_month));
    Ok((target_year as i32, target_month, target_day))
}
pub(crate) fn days_in_year_to_date(year: i32, month: u8, day: u8) -> u16 {
    let jan1 = days_from_civil(year, 1, 1);
    (days_from_civil(year, u32::from(month), u32::from(day)) - jan1 + 1) as u16
}
pub(crate) fn weeks_in_year_start(year: i32, first_day: u8, mode: u8) -> i64 {
    let jan1 = days_from_civil(year, 1, 1) - OA_EPOCH_UNIX_DAYS;
    let jan1_weekday = i64::from(weekday_sunday_one(jan1));
    let first = i64::from(first_day);
    let back = (jan1_weekday - first).rem_euclid(7);
    match mode {
        2 => jan1 - back + i64::from(back > 3) * 7,
        3 => jan1 - back + 7,
        _ => jan1 - back,
    }
}
