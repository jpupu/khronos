use chrono::{Duration, NaiveDateTime};

#[derive(Debug, PartialEq, Eq)]
pub enum InputFormat {
    /// Seconds since midnight 1970-01-01
    Unix,
    /// Milliseconds since midnight 1970-01-01
    UnixMs,
    /// E.g. "%Y-%m-%d %H:%M". Date, hour and minute fields are mandatory.
    Epoc(NaiveDateTime),
    Iso8601,
    Custom(String),
}

/// Parses a decimal number into integer and nano parts.
fn parse_decimal(s: &str) -> Option<(i64, u32)> {
    Some(match s.find('.') {
        Some(i) => (s[..i].parse::<i64>().ok()?, {
            // Parse at most 9 digits after the decimal point.
            let f = &s[(i + 1)..(s.len().min(i + 10))];
            let n = f.parse::<u32>().ok()?;
            n * 10u32.pow(9 - f.len() as u32)
        }),
        None => (s.parse::<i64>().ok()?, 0),
    })
}

/// Parses string to datetime according to given format.
pub fn parse_string(s: &str, format: InputFormat) -> Option<NaiveDateTime> {
    Some(match format {
        InputFormat::Unix => {
            let (sec, nsec) = parse_decimal(s)?;
            NaiveDateTime::from_timestamp(sec, nsec)
        }
        InputFormat::UnixMs => {
            let (msec, psec) = parse_decimal(s)?;
            NaiveDateTime::from_timestamp(
                msec / 1000,
                (msec % 1000) as u32 * 1_000_000 + psec / 1000,
            )
        }
        InputFormat::Epoc(epoc) => {
            let (sec, nsec) = parse_decimal(s)?;
            epoc + Duration::seconds(sec) + Duration::nanoseconds(nsec.into())
        }
        InputFormat::Iso8601 => NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").ok()?,
        InputFormat::Custom(fmt) => NaiveDateTime::parse_from_str(s, &fmt).ok()?,
    })
}

/// Parses line to timestamp and remainder.
///
/// Assumes the timestamp is in the beginning of the line, does not contain whitespace (space or
/// tab), and is followed by whitespace. This whitespace is included in the remainder.
///
/// If timestamp cannot be parsed, returns None as timestamp and the whole line as the remainder.
pub fn parse_line(s: &str, format: InputFormat) -> (Option<NaiveDateTime>, &str) {
    match s.find(&[' ', '\t']) {
        Some(i) => match parse_string(&s[..i], format) {
            Some(timestamp) => (Some(timestamp), &s[i..]),
            None => (None, s),
        },
        None => (None, s),
    }
}

/// Tries to automatically detect the timestamp format used.
///
/// Assumes the timestamp is in the beginning of the line, does not contain whitespace (space or
/// tab), and is followed by whitespace.
pub fn detect_format(s: &str) -> Option<InputFormat> {
    let ts = &s[..s.find(&[' ', '\t'])?];

    if NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.f").is_ok() {
        return Some(InputFormat::Iso8601);
    }

    // 100 billion is 1973-03-03 in if interpreted as milliseconds, 5138-11-16 if interpreted in
    // seconds. So it's reasonable to assume any bigger timestamps are in milliseconds.
    match parse_decimal(ts) {
        Some((x, _)) if x > 100_000_000_000 => return Some(InputFormat::UnixMs),
        Some(_) => return Some(InputFormat::Unix),
        None => (),
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_decimal_with_nano() {
        assert_eq!(parse_decimal("123"), Some((123, 0)));
        assert_eq!(parse_decimal("123.001002003"), Some((123, 1002003)));
        assert_eq!(parse_decimal("123.0010020039"), Some((123, 1002003)));
        assert_eq!(parse_decimal("123.001002"), Some((123, 1002000)));
        assert_eq!(parse_decimal("123.1"), Some((123, 100000000)));
        assert_eq!(parse_decimal("123.1foo"), None);
        assert_eq!(parse_decimal("foo.123"), None);
    }

    #[test]
    fn test_parse_string_unix() {
        assert_eq!(
            parse_string("1000", InputFormat::Unix),
            Some(NaiveDateTime::from_timestamp(1000, 0))
        );
        assert_eq!(
            parse_string("1000.000123456", InputFormat::Unix),
            Some(NaiveDateTime::from_timestamp(1000, 123456))
        );
        assert_eq!(parse_string("abc", InputFormat::Unix), None);
    }

    #[test]
    fn test_parse_string_unixms() {
        assert_eq!(
            parse_string("1234", InputFormat::UnixMs),
            Some(NaiveDateTime::from_timestamp(1, 234_000_000))
        );
        assert_eq!(
            parse_string("1000.000123456", InputFormat::UnixMs),
            Some(NaiveDateTime::from_timestamp(1, 123))
        );
        assert_eq!(parse_string("abc", InputFormat::UnixMs), None);
    }

    #[test]
    fn test_parse_string_epoc() {
        let epoc = NaiveDateTime::new(
            NaiveDate::from_ymd(2000, 1, 1),
            NaiveTime::from_hms(0, 0, 0),
        );
        assert_eq!(
            parse_string("86460", InputFormat::Epoc(epoc)),
            Some(epoc + Duration::days(1) + Duration::minutes(1))
        );
        assert_eq!(
            parse_string("86460.001", InputFormat::Epoc(epoc)),
            Some(epoc + Duration::days(1) + Duration::minutes(1) + Duration::milliseconds(1))
        );
        assert_eq!(parse_string("abc", InputFormat::Epoc(epoc)), None);
    }

    #[test]
    fn test_parse_string_custom() {
        assert_eq!(
            parse_string(
                "2001-02-13 12:34",
                InputFormat::Custom("%Y-%m-%d %H:%M".to_string())
            ),
            Some(NaiveDateTime::new(
                NaiveDate::from_ymd(2001, 2, 13),
                NaiveTime::from_hms(12, 34, 0)
            ))
        );
        assert_eq!(
            parse_string(
                "2001-02-13 12:34:56.123456",
                InputFormat::Custom("%Y-%m-%d %H:%M:%S%.f".to_string())
            ),
            Some(NaiveDateTime::new(
                NaiveDate::from_ymd(2001, 2, 13),
                NaiveTime::from_hms_micro(12, 34, 56, 123456)
            ))
        );
        assert_eq!(
            parse_string(
                "2001x02x13 12x34",
                InputFormat::Custom("%Y-%m-%d %H:%M".to_string())
            ),
            None
        );
        assert_eq!(
            parse_string(
                "2001x02x13",
                InputFormat::Custom("%Y-%m-%d %H:%M".to_string())
            ),
            None
        );
    }

    #[test]
    fn test_parse_string_iso8601() {
        // With milliseconds
        assert_eq!(
            parse_string("2001-02-13T12:34:56.123", InputFormat::Iso8601),
            Some(NaiveDateTime::new(
                NaiveDate::from_ymd(2001, 2, 13),
                NaiveTime::from_hms_milli(12, 34, 56, 123)
            ))
        );
        // With nanoseconds
        assert_eq!(
            parse_string("2001-02-13T12:34:56.123456789", InputFormat::Iso8601),
            Some(NaiveDateTime::new(
                NaiveDate::from_ymd(2001, 2, 13),
                NaiveTime::from_hms_nano(12, 34, 56, 123456789)
            ))
        );
        // No fractional seconds
        assert_eq!(
            parse_string("2001-02-13T12:34:56", InputFormat::Iso8601),
            Some(NaiveDateTime::new(
                NaiveDate::from_ymd(2001, 2, 13),
                NaiveTime::from_hms(12, 34, 56)
            ))
        );
        // Space as date-time separator.
        assert_eq!(
            parse_string("2001-02-13 12:34:56", InputFormat::Iso8601),
            None
        );
    }

    #[test]
    fn test_parse_line() {
        // Space separator
        assert_eq!(
            parse_line("123.4 Log message", InputFormat::Unix),
            (
                Some(NaiveDateTime::from_timestamp(123, 400_000_000)),
                " Log message"
            )
        );
        // Tab separator
        assert_eq!(
            parse_line("123.4\tLog message", InputFormat::Unix),
            (
                Some(NaiveDateTime::from_timestamp(123, 400_000_000)),
                "\tLog message"
            )
        );
        // No timestamp, message contains separator.
        assert_eq!(
            parse_line("Log message", InputFormat::Unix),
            (None, "Log message")
        );
        // No whitespace
        assert_eq!(
            parse_line("Logmessage", InputFormat::Unix),
            (None, "Logmessage")
        );
        // Start with space
        assert_eq!(
            parse_line(" Logmessage", InputFormat::Unix),
            (None, " Logmessage")
        );
        // Empty
        assert_eq!(parse_line("", InputFormat::Unix), (None, ""));
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(
            detect_format("982240496.123 Log message"),
            Some(InputFormat::Unix)
        );
        assert_eq!(
            detect_format("1650400500.123 Log message"),
            Some(InputFormat::Unix)
        );
        assert_eq!(
            detect_format("982240496123.456 Log message"),
            Some(InputFormat::UnixMs)
        );
        assert_eq!(
            detect_format("1650400500123.456 Log message"),
            Some(InputFormat::UnixMs)
        );
        assert_eq!(
            detect_format("2001-12-13T12:34:56 Log message"),
            Some(InputFormat::Iso8601)
        );
        assert_eq!(
            detect_format("2001-12-13T12:34:56.123 Log message"),
            Some(InputFormat::Iso8601)
        );
        assert_eq!(detect_format("Log message"), None);
        assert_eq!(detect_format("Logmessage"), None);
        assert_eq!(detect_format(" Logmessage"), None);
        assert_eq!(detect_format(" "), None);
        assert_eq!(detect_format(""), None);
    }
}
