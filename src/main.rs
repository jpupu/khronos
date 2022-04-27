use clap::Parser;
use khronos::{self, InputFormat, OutputFormat, Unit};
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
struct Args {
    #[clap(parse(try_from_str=parse_output_format))]
    outformat: OutputFormat,
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

fn process_text<R, F>(informat: InputFormat, outformat: OutputFormat, input: R, mut func: F)
where
    R: BufRead,
    F: FnMut(&str, &str),
{
    let mut prev_intime = None;
    for line in input.lines().map(|x| x.expect("line error")) {
        let (intime, text) = khronos::parse_line(&line, informat);
        let outtime = match intime {
            Some(t) => khronos::write(outformat, t, prev_intime),
            None => "".to_string(),
        };
        prev_intime = intime;

        func(&outtime, text);
    }
}

fn main() {
    let args = Args::parse();
    let informat = InputFormat::Unix;

    process_text(
        informat,
        args.outformat,
        io::stdin().lock(),
        |time, text| println!("{}{}", time, text),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_process_text(
        informat: InputFormat,
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
            InputFormat::Unix,
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
            InputFormat::Unix,
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
