//! A calendar date: the as-of date and every deadline (SR-BASELINE
//! decision 4). Nothing here reads the clock (L5 decision 2).

use std::fmt;

/// A proleptic Gregorian date, `YYYY-MM-DD`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

fn leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i32, m: u8) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap(y) => 29,
        _ => 28,
    }
}

impl Date {
    /// `YYYY-MM-DD`, exactly; a date-time is refused, so a deadline is
    /// never read in some time zone.
    pub fn parse(s: &str) -> Result<Date, String> {
        let bad = || format!("`{s}` is not a date; write it as YYYY-MM-DD");
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return Err(bad());
        }
        let num = |r: std::ops::Range<usize>| s.get(r).filter(|p| p.bytes().all(|c| c.is_ascii_digit())).and_then(|p| p.parse::<u32>().ok());
        let (Some(y), Some(m), Some(d)) = (num(0..4), num(5..7), num(8..10)) else { return Err(bad()) };
        let (year, month, day) = (y as i32, m as u8, d as u8);
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return Err(bad());
        }
        Ok(Date { year, month, day })
    }

    /// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
    pub fn days(self) -> i64 {
        let y = i64::from(self.year) - i64::from(self.month <= 2);
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = i64::from(self.month);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    /// The date `days` after 1970-01-01.
    pub fn from_days(days: i64) -> Date {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = (yoe + era * 400 + i64::from(month <= 2)) as i32;
        Date { year, month, day }
    }

    /// Days from `self` to `later` (negative when `later` is earlier).
    pub fn days_until(self, later: Date) -> i64 {
        later.days() - self.days()
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A date from a TOML value: a string `"2027-03-31"` or a bare TOML date.
pub fn from_toml(v: &toml::Value) -> Result<Date, String> {
    match v {
        toml::Value::String(s) => Date::parse(s),
        toml::Value::Datetime(dt) if dt.time.is_none() && dt.offset.is_none() => Date::parse(&dt.to_string()),
        other => Err(format!("`{other}` is not a date; write it as \"YYYY-MM-DD\"")),
    }
}
