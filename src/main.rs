use khronos::{self, InputFormat, OutputFormat};
use std::io::{self, BufRead};

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
    let informat = InputFormat::Unix;
    let outformat = OutputFormat::Iso8601;

    process_text(informat, outformat, io::stdin().lock(), |time, text| {
        println!("{}{}", time, text)
    });
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
}
