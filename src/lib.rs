use chrono::{Duration, NaiveDateTime};

pub enum InputFormat {
    Unix,
    Epoc(NaiveDateTime),
    /// E.g. "%Y-%m-%d %H:%M". Date, hour and minute fields are mandatory.
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

pub fn parse_string(s: &str, format: InputFormat) -> Option<NaiveDateTime> {
    Some(match format {
        InputFormat::Unix => {
            let (sec, nsec) = parse_decimal(s)?;
            NaiveDateTime::from_timestamp(sec, nsec)
        }
        InputFormat::Epoc(epoc) => {
            let (sec, nsec) = parse_decimal(s)?;
            epoc + Duration::seconds(sec) + Duration::nanoseconds(nsec.into())
        }
        InputFormat::Custom(fmt) => NaiveDateTime::parse_from_str(s, &fmt).ok()?,
    })
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
}
