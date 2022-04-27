use chrono::NaiveDateTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Iso8601,
    Unix(Unit),
    Delta(Unit),
}

pub fn write(format: OutputFormat, t: NaiveDateTime, prev_t: Option<NaiveDateTime>) -> String {
    match format {
        OutputFormat::Iso8601 => t.format("%Y-%m-%dT%H:%M:%S").to_string(),
        OutputFormat::Unix(Unit::Seconds) => t.timestamp().to_string(),
        OutputFormat::Unix(Unit::Milliseconds) => t.timestamp_millis().to_string(),
        OutputFormat::Unix(Unit::Microseconds) => (t.timestamp_nanos() / 1000).to_string(),
        OutputFormat::Unix(Unit::Nanoseconds) => t.timestamp_nanos().to_string(),
        OutputFormat::Delta(Unit::Seconds) => (t - prev_t.unwrap_or(t)).num_seconds().to_string(),
        OutputFormat::Delta(Unit::Milliseconds) => {
            (t - prev_t.unwrap_or(t)).num_milliseconds().to_string()
        }
        OutputFormat::Delta(Unit::Microseconds) => (t - prev_t.unwrap_or(t))
            .num_microseconds()
            .unwrap_or(0)
            .to_string(),
        OutputFormat::Delta(Unit::Nanoseconds) => (t - prev_t.unwrap_or(t))
            .num_nanoseconds()
            .unwrap_or(0)
            .to_string(),
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
            write(OutputFormat::Iso8601, some_date(), None),
            "2001-02-15T12:34:56"
        );
    }

    #[test]
    fn output_unix() {
        assert_eq!(
            write(OutputFormat::Unix(Unit::Seconds), some_date(), None),
            "982240496"
        );
        assert_eq!(
            write(OutputFormat::Unix(Unit::Milliseconds), some_date(), None),
            "982240496123"
        );
        assert_eq!(
            write(OutputFormat::Unix(Unit::Microseconds), some_date(), None),
            "982240496123456"
        );
        assert_eq!(
            write(OutputFormat::Unix(Unit::Nanoseconds), some_date(), None),
            "982240496123456789"
        );
    }

    #[test]
    fn output_delta() {
        assert_eq!(
            write(OutputFormat::Delta(Unit::Seconds), some_date(), None),
            "0"
        );
        assert_eq!(
            write(
                OutputFormat::Delta(Unit::Seconds),
                some_date(),
                Some(some_date() - Duration::seconds(130)),
            ),
            "130"
        );
        assert_eq!(
            write(
                OutputFormat::Delta(Unit::Milliseconds),
                some_date(),
                Some(some_date() - Duration::milliseconds(130)),
            ),
            "130"
        );
        assert_eq!(
            write(
                OutputFormat::Delta(Unit::Microseconds),
                some_date(),
                Some(some_date() - Duration::microseconds(130)),
            ),
            "130"
        );
        assert_eq!(
            write(
                OutputFormat::Delta(Unit::Nanoseconds),
                some_date(),
                Some(some_date() - Duration::nanoseconds(130)),
            ),
            "130"
        );
    }
}
