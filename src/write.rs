use chrono::NaiveDateTime;

pub enum OutputFormat {
    Iso8601,
    Unix,
    Delta,
}

pub fn write(format: OutputFormat, t: NaiveDateTime, prev_t: Option<NaiveDateTime>) -> String {
    match format {
        OutputFormat::Iso8601 => t.format("%Y-%m-%dT%H:%M:%S").to_string(),
        OutputFormat::Unix => t.timestamp().to_string(),
        OutputFormat::Delta => (t - prev_t.unwrap_or(t)).num_seconds().to_string(),
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
        assert_eq!(write(OutputFormat::Unix, some_date(), None), "982240496");
    }

    #[test]
    fn output_delta() {
        assert_eq!(write(OutputFormat::Delta, some_date(), None), "0");
        assert_eq!(
            write(
                OutputFormat::Delta,
                some_date(),
                Some(some_date() - Duration::seconds(130)),
            ),
            "130"
        );
    }
}
