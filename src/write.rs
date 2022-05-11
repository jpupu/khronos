use chrono::NaiveDateTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Seconds = 0,
    Milliseconds = 1,
    Microseconds = 2,
    Nanoseconds = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Precision(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Iso8601 { prec: Precision, time_only: bool },
    Unix(Unit, Precision),
    Delta(Unit, Precision),
}

fn format_seconds(seconds: i64, nanos: u32, units: Unit, prec: Precision) -> String {
    let prec = prec.0 as u32;
    let nanos = nanos as i64;
    assert!(prec <= 9);

    let mag = 1000i64.pow(units as u32);
    let rmag = 1000i64.pow(3 - units as u32);
    let full = seconds as u128 * mag as u128 + (nanos / rmag) as u128;
    let frac = nanos % rmag;
    let frac_digits = 9 - units as u32 * 3;

    if prec == 0 {
        format!("{}", full)
    } else if frac_digits > prec {
        format!(
            "{}.{:0width$}",
            full,
            frac / 10i64.pow(frac_digits - prec),
            width = prec as usize
        )
    } else {
        format!(
            "{}.{:0width$}",
            full,
            frac * 10i64.pow(prec - frac_digits),
            width = prec as usize
        )
    }
}

pub fn write(format: OutputFormat, t: NaiveDateTime, prev_t: Option<NaiveDateTime>) -> String {
    match format {
        OutputFormat::Iso8601 { prec, time_only } => {
            let mut s = t
                .format(match time_only {
                    false => "%Y-%m-%dT%H:%M:%S%.9f",
                    true => "%H:%M:%S%.9f",
                })
                .to_string();
            match prec {
                Precision(0) => s.truncate(s.len() - 10),
                Precision(n) => s.truncate(s.len() - 9 + n),
            }
            s
        }
        OutputFormat::Unix(unit, prec) => {
            format_seconds(t.timestamp(), t.timestamp_subsec_nanos(), unit, prec)
        }
        OutputFormat::Delta(unit, prec) => {
            let ns = (t - prev_t.unwrap_or(t))
                .num_nanoseconds()
                .expect("Too large delta");
            format_seconds(ns / 1_000_000_000, (ns % 1_000_000_000) as u32, unit, prec)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, NaiveDate, NaiveTime};

    fn some_date() -> NaiveDateTime {
        NaiveDateTime::new(
            NaiveDate::from_ymd(2001, 2, 15),
            NaiveTime::from_hms_nano(12, 34, 56, 123_456_789),
        )
    }

    #[test]
    fn output_iso8601() {
        assert_eq!(
            write(
                OutputFormat::Iso8601 {
                    prec: Precision(0),
                    time_only: false
                },
                some_date(),
                None
            ),
            "2001-02-15T12:34:56"
        );
        assert_eq!(
            write(
                OutputFormat::Iso8601 {
                    prec: Precision(1),
                    time_only: false
                },
                some_date(),
                None
            ),
            "2001-02-15T12:34:56.1"
        );
        assert_eq!(
            write(
                OutputFormat::Iso8601 {
                    prec: Precision(3),
                    time_only: false
                },
                some_date(),
                None
            ),
            "2001-02-15T12:34:56.123"
        );
        assert_eq!(
            write(
                OutputFormat::Iso8601 {
                    prec: Precision(0),
                    time_only: true
                },
                some_date(),
                None
            ),
            "12:34:56"
        );
        assert_eq!(
            write(
                OutputFormat::Iso8601 {
                    prec: Precision(3),
                    time_only: true
                },
                some_date(),
                None
            ),
            "12:34:56.123"
        );
    }

    #[test]
    fn output_unix() {
        assert_eq!(
            write(
                OutputFormat::Unix(Unit::Seconds, Precision(0)),
                some_date(),
                None
            ),
            "982240496"
        );
        assert_eq!(
            write(
                OutputFormat::Unix(Unit::Milliseconds, Precision(0)),
                some_date(),
                None
            ),
            "982240496123"
        );
        assert_eq!(
            write(
                OutputFormat::Unix(Unit::Microseconds, Precision(3)),
                some_date(),
                None
            ),
            "982240496123456.789"
        );
        assert_eq!(
            write(
                OutputFormat::Unix(Unit::Nanoseconds, Precision(9)),
                some_date(),
                None
            ),
            "982240496123456789.000000000"
        );
    }

    #[test]
    fn output_delta() {
        assert_eq!(
            write(
                OutputFormat::Delta(Unit::Seconds, Precision(0)),
                some_date(),
                None
            ),
            "0"
        );
        assert_eq!(
            write(
                OutputFormat::Delta(Unit::Seconds, Precision(0)),
                some_date(),
                Some(some_date() - Duration::seconds(130)),
            ),
            "130"
        );
        assert_eq!(
            write(
                OutputFormat::Delta(Unit::Milliseconds, Precision(0)),
                some_date(),
                Some(some_date() - Duration::milliseconds(130)),
            ),
            "130"
        );
        assert_eq!(
            write(
                OutputFormat::Delta(Unit::Microseconds, Precision(0)),
                some_date(),
                Some(some_date() - Duration::microseconds(130)),
            ),
            "130"
        );
        assert_eq!(
            write(
                OutputFormat::Delta(Unit::Nanoseconds, Precision(3)),
                some_date(),
                Some(some_date() - Duration::nanoseconds(130)),
            ),
            "130.000"
        );
    }

    mod test_format_seconds {
        use super::*;

        #[test]
        fn integer_seconds() {
            assert_eq!("0", format_seconds(0, 0, Unit::Seconds, Precision(0)));
            assert_eq!("123", format_seconds(123, 0, Unit::Seconds, Precision(0)));
            assert_eq!(
                "42",
                format_seconds(42, 123_456_789, Unit::Seconds, Precision(0))
            );
            assert_eq!("0", format_seconds(0, 456_000, Unit::Seconds, Precision(0)));
            assert_eq!(
                "10000000000",
                format_seconds(10_000_000_000, 123_456_789, Unit::Seconds, Precision(0))
            );
        }

        #[test]
        fn integer_microseconds() {
            assert_eq!("0", format_seconds(0, 0, Unit::Microseconds, Precision(0)));
            assert_eq!(
                "123000000",
                format_seconds(123, 0, Unit::Microseconds, Precision(0))
            );
            assert_eq!(
                "456",
                format_seconds(0, 456_000, Unit::Microseconds, Precision(0))
            );
            assert_eq!(
                "42123456",
                format_seconds(42, 123_456_789, Unit::Microseconds, Precision(0))
            );
        }

        #[test]
        fn fractional_seconds() {
            assert_eq!("0.0", format_seconds(0, 0, Unit::Seconds, Precision(1)));
            assert_eq!("0.000", format_seconds(0, 0, Unit::Seconds, Precision(3)));
            assert_eq!(
                "0.000000000",
                format_seconds(0, 0, Unit::Seconds, Precision(9))
            );
            assert_eq!(
                "42.123",
                format_seconds(42, 123_456_789, Unit::Seconds, Precision(3))
            );
            assert_eq!(
                "42.123456789",
                format_seconds(42, 123_456_789, Unit::Seconds, Precision(9))
            );
            assert_eq!(
                "0.000456",
                format_seconds(0, 456_000, Unit::Seconds, Precision(6))
            );
            assert_eq!(
                "10000000000.123456789",
                format_seconds(10_000_000_000, 123_456_789, Unit::Seconds, Precision(9))
            );
        }

        #[test]
        fn fractional_microseconds() {
            assert_eq!(
                "0.0",
                format_seconds(0, 0, Unit::Microseconds, Precision(1))
            );
            assert_eq!(
                "0.000",
                format_seconds(0, 0, Unit::Microseconds, Precision(3))
            );
            assert_eq!(
                "0.000000000",
                format_seconds(0, 0, Unit::Microseconds, Precision(9))
            );
            assert_eq!(
                "10000000000123456.789000000",
                format_seconds(
                    10_000_000_000,
                    123_456_789,
                    Unit::Microseconds,
                    Precision(9)
                )
            );
        }

        #[test]
        fn fractional_nanoseconds() {
            assert_eq!(
                "10000000000123456789.000000000",
                format_seconds(10_000_000_000, 123_456_789, Unit::Nanoseconds, Precision(9))
            );
        }
    }
}
