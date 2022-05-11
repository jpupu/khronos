use clap::Parser;
use khronos::{self, InputFormat, OutputFormat, Precision, Unit};
use std::io::{self, BufRead};

/// Log timestamp rewriter
///
/// Reads from stdin lines and rewrites their timestamps. The timestamp must
/// be at the start of line and separated from the message by at least one
/// space. If the timestamp of a line cannot be successfully parsed, the line
/// is output as-is.
///
/// If input format is not given it is automatically deduced from input.
/// In this case the lines are read and output as-is until the first
/// recognizable timestamp is met.
#[derive(Parser, Debug)]
#[clap(after_help = r"INPUT FORMATS:
    iso     ISO 8601
    unix    Unix time in (fractional) seconds
    unixms  Unix time in (fractional) milliseconds

OUTPUT FORMATS:
    iso     ISO 8601. Options: precision, nodate
    unix    Unix time. Options: units, precision
    delta   Time since previous line. Options: units, precision

OUTPUT OPTIONS:
    precision   .0 | .1 | .2 | ... | .9
    units       s | ms | us | ns
    nodate      nodate

EXAMPLES:
    Specify unix time in milliseconds with 3 fractional digits:
        unix,ms,.3

    Specify delta in seconds with 6 fractional digits:
        delta,.6
")]
struct Args {
    /// Input format. Auto-detect if not specified.
    #[clap(
        short,
        long,
        value_name="FMT",
        parse(try_from_str=parse_input_format),
    )]
    informat: Option<InputFormat>,

    /// Output format.
    #[clap(short,
        long,
        value_name="FMT[,OPTION...]",
        default_value="iso",
        parse(try_from_str=parse_output_format),
    )]
    outformat: OutputFormat,
}

fn parse_input_format(s: &str) -> Result<InputFormat, String> {
    match s {
        "unix" => Ok(InputFormat::Unix),
        "unixms" => Ok(InputFormat::UnixMs),
        "iso" => Ok(InputFormat::Iso8601),
        _ => Err("Invalid format".to_string()),
    }
}

fn try_parse_unit(s: &str) -> Option<Unit> {
    match s {
        "s" => Some(Unit::Seconds),
        "ms" => Some(Unit::Milliseconds),
        "us" => Some(Unit::Microseconds),
        "ns" => Some(Unit::Nanoseconds),
        _ => None,
    }
}

fn try_parse_precision(s: &str) -> Option<Precision> {
    if s.starts_with(".") {
        s[1..]
            .parse()
            .ok()
            .filter(|x| *x <= 9)
            .map(|x| Precision(x))
    } else {
        None
    }
}

fn parse_output_format(s: &str) -> Result<OutputFormat, String> {
    let args = s.split(',').collect::<Vec<&str>>();
    let (fmt, args) = args.split_first().unwrap();
    match *fmt {
        "iso" => {
            let mut prec = Precision(0);
            let mut time_only = false;
            for a in args {
                if let Some(p) = try_parse_precision(a) {
                    prec = p;
                } else if *a == "nodate" {
                    time_only = true;
                } else {
                    return Err(format!("Invalid format argument {:?}", a));
                }
            }
            Ok(OutputFormat::Iso8601 { prec, time_only })
        }
        "unix" => {
            let mut unit = Unit::Seconds;
            let mut prec = Precision(0);
            for a in args {
                if let Some(u) = try_parse_unit(a) {
                    unit = u;
                } else if let Some(p) = try_parse_precision(a) {
                    prec = p;
                } else {
                    return Err(format!("Invalid format argument {:?}", a));
                }
            }
            Ok(OutputFormat::Unix(unit, prec))
        }
        "delta" => {
            let mut unit = Unit::Seconds;
            let mut prec = Precision(0);
            for a in args {
                if let Some(u) = try_parse_unit(a) {
                    unit = u;
                } else if let Some(p) = try_parse_precision(a) {
                    prec = p;
                } else {
                    return Err(format!("Invalid format argument {:?}", a));
                }
            }
            Ok(OutputFormat::Delta(unit, prec))
        }
        _ => Err("Invalid output format".to_string()),
    }
}

fn process_text<R, F>(
    mut informat: Option<InputFormat>,
    outformat: OutputFormat,
    input: R,
    mut func: F,
) where
    R: BufRead,
    F: FnMut(&str, &str),
{
    let mut prev_intime = None;
    for line in input.lines().map(|x| x.expect("line error")) {
        // Try to auto-detect input format if it's not known.
        if informat.is_none() {
            informat = khronos::detect_format(&line);
        }

        // Process line.
        if let Some(ref fmt) = informat {
            let (intime, text) = khronos::parse_line(&line, fmt);
            let outtime = match intime {
                Some(t) => khronos::write(outformat, t, prev_intime),
                None => "".to_string(),
            };
            prev_intime = intime;
            func(&outtime, text);
        } else {
            func("", &line);
        }
    }
}

fn main() {
    let args = Args::parse();

    process_text(
        args.informat,
        args.outformat,
        io::stdin().lock(),
        |time, text| println!("{}{}", time, text),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_process_text(
        informat: Option<InputFormat>,
        outformat: OutputFormat,
        input: &str,
        expected_output: Vec<(&str, &str)>,
    ) {
        let cursor = io::Cursor::new(input);
        let mut expected_iter = expected_output.iter();
        process_text(informat, outformat, cursor, |time, text| {
            assert_eq!(
                &(time, text),
                expected_iter.next().expect("produced too many lines")
            )
        });
    }

    #[test]
    fn basic() {
        check_process_text(
            Some(InputFormat::Unix),
            OutputFormat::Iso8601 {
                prec: Precision(0),
                time_only: false,
            },
            "000.0 a line\n60.66 another line\n",
            vec![
                ("1970-01-01T00:00:00", " a line"),
                ("1970-01-01T00:01:00", " another line"),
            ],
        );
    }

    #[test]
    fn no_timestamp() {
        check_process_text(
            Some(InputFormat::Unix),
            OutputFormat::Iso8601 {
                prec: Precision(0),
                time_only: false,
            },
            "000.0 a line\nanother line\n\n",
            vec![
                ("1970-01-01T00:00:00", " a line"),
                ("", "another line"),
                ("", ""),
            ],
        );
    }

    #[test]
    fn auto_detect_input_format_from_first_line() {
        check_process_text(
            None,
            OutputFormat::Iso8601 {
                prec: Precision(0),
                time_only: false,
            },
            "000.0 a line\n60.66 another line\n",
            vec![
                ("1970-01-01T00:00:00", " a line"),
                ("1970-01-01T00:01:00", " another line"),
            ],
        );
    }

    #[test]
    fn auto_detect_input_format_from_second_line() {
        check_process_text(
            None,
            OutputFormat::Iso8601 {
                prec: Precision(0),
                time_only: false,
            },
            "notime\nstillno\n000.0 a line\n60.66 another line\n",
            vec![
                ("", "notime"),
                ("", "stillno"),
                ("1970-01-01T00:00:00", " a line"),
                ("1970-01-01T00:01:00", " another line"),
            ],
        );
    }

    #[test]
    fn verify_app() {
        use clap::CommandFactory;
        Args::command().debug_assert();
    }

    #[test]
    fn test_parse_output_format_iso8601() {
        assert_eq!(
            parse_output_format("iso"),
            Ok(OutputFormat::Iso8601 {
                prec: Precision(0),
                time_only: false
            })
        );
        assert_eq!(
            parse_output_format("iso,.1"),
            Ok(OutputFormat::Iso8601 {
                prec: Precision(1),
                time_only: false
            })
        );
        assert_eq!(
            parse_output_format("iso,.3"),
            Ok(OutputFormat::Iso8601 {
                prec: Precision(3),
                time_only: false
            })
        );
        assert_eq!(
            parse_output_format("iso,.3,nodate"),
            Ok(OutputFormat::Iso8601 {
                prec: Precision(3),
                time_only: true
            })
        );
    }

    #[test]
    fn test_parse_output_format_unix() {
        assert_eq!(
            parse_output_format("unix"),
            Ok(OutputFormat::Unix(Unit::Seconds, Precision(0)))
        );
        assert_eq!(
            parse_output_format("unix,s"),
            Ok(OutputFormat::Unix(Unit::Seconds, Precision(0)))
        );
        assert_eq!(
            parse_output_format("unix,ms"),
            Ok(OutputFormat::Unix(Unit::Milliseconds, Precision(0)))
        );
        assert_eq!(
            parse_output_format("unix,us"),
            Ok(OutputFormat::Unix(Unit::Microseconds, Precision(0)))
        );
        assert_eq!(
            parse_output_format("unix,ns"),
            Ok(OutputFormat::Unix(Unit::Nanoseconds, Precision(0)))
        );
        assert_eq!(
            parse_output_format("unix,.3"),
            Ok(OutputFormat::Unix(Unit::Seconds, Precision(3)))
        );
        assert_eq!(
            parse_output_format("unix,ns,.1"),
            Ok(OutputFormat::Unix(Unit::Nanoseconds, Precision(1)))
        );
        assert_eq!(
            parse_output_format("unix,.1,ns"),
            Ok(OutputFormat::Unix(Unit::Nanoseconds, Precision(1)))
        );
    }

    #[test]
    fn test_parse_output_format_delta() {
        assert_eq!(
            parse_output_format("delta,ms"),
            Ok(OutputFormat::Delta(Unit::Milliseconds, Precision(0)))
        );
        assert_eq!(
            parse_output_format("delta,.9"),
            Ok(OutputFormat::Delta(Unit::Seconds, Precision(9)))
        );
    }
}
