use clap::Parser;
use khronos::{self, InputFormat, OutputFormat, Unit};
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
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
        value_name="FMT",
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

fn parse_output_format(s: &str) -> Result<OutputFormat, String> {
    let args = s.split(',').collect::<Vec<&str>>();
    let (fmt, args) = args.split_first().unwrap();
    match *fmt {
        "iso" => Ok(OutputFormat::Iso8601),
        "unix" => {
            let mut unit = Unit::Seconds;
            for a in args {
                match *a {
                    "s" => unit = Unit::Seconds,
                    "ms" => unit = Unit::Milliseconds,
                    "us" => unit = Unit::Microseconds,
                    "ns" => unit = Unit::Nanoseconds,
                    _ => return Err(format!("Invalid format argument {:?}", a)),
                };
            }
            Ok(OutputFormat::Unix(unit))
        }
        "delta" => {
            let mut unit = Unit::Seconds;
            for a in args {
                match *a {
                    "s" => unit = Unit::Seconds,
                    "ms" => unit = Unit::Milliseconds,
                    "us" => unit = Unit::Microseconds,
                    "ns" => unit = Unit::Nanoseconds,
                    _ => return Err(format!("Invalid format argument {:?}", a)),
                };
            }
            Ok(OutputFormat::Delta(unit))
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
            OutputFormat::Iso8601,
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
            OutputFormat::Iso8601,
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
            OutputFormat::Iso8601,
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
            OutputFormat::Iso8601,
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
        assert_eq!(parse_output_format("iso"), Ok(OutputFormat::Iso8601));
    }

    #[test]
    fn test_parse_output_format_unix() {
        assert_eq!(
            parse_output_format("unix"),
            Ok(OutputFormat::Unix(Unit::Seconds))
        );
        assert_eq!(
            parse_output_format("unix,s"),
            Ok(OutputFormat::Unix(Unit::Seconds))
        );
        assert_eq!(
            parse_output_format("unix,ms"),
            Ok(OutputFormat::Unix(Unit::Milliseconds))
        );
        assert_eq!(
            parse_output_format("unix,us"),
            Ok(OutputFormat::Unix(Unit::Microseconds))
        );
        assert_eq!(
            parse_output_format("unix,ns"),
            Ok(OutputFormat::Unix(Unit::Nanoseconds))
        );
    }

    #[test]
    fn test_parse_output_format_delta() {
        assert_eq!(
            parse_output_format("delta,ms"),
            Ok(OutputFormat::Delta(Unit::Milliseconds))
        );
    }
}
