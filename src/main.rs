use anyhow::{Context, Result};
use chrono::{Duration, NaiveTime};
use clap::Parser;
use std::{
    fs,
    fs::File,
    io::{BufRead, BufReader, Write},
};
use thiserror::Error;

#[derive(Parser)]
#[clap(
    version = "0.1.0",
    author = "abhayk <abhay.krishnan.dev@gmail.com>",
    about = r#"A tool to add or subtract offsets to the timestamps in a .srt subtitle file. 
    After offsets are applied the original file will be backed up to <file>.orig"#
)]
struct Opts {
    #[clap(short, long, about = "The path to the subtitle file")]
    file: String,
    #[clap(
        short,
        long,
        about = "The shift offset. To increment by half a second provide +500, To decrement -500.",
        allow_hyphen_values = true
    )]
    offset: i8,
}

#[derive(Error, Debug, PartialEq)]
enum CustomError {
    #[error("An error occurred while parsing `{value}`: {source_error}")]
    ParseError { value: String, source_error: String },
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let output_file_path = opts.file.clone() + ".tmp";
    process_file(&opts.file, &output_file_path, opts.offset)
}

fn process_file(input_file_path: &str, output_file_path: &str, offset: i8) -> Result<()> {
    let input_file = File::open(input_file_path)
        .with_context(|| format!("Could not read the file `{}`", input_file_path))?;
    let mut output_file =
        File::create(output_file_path).with_context(|| "Could not create the output file.")?;

    let buffered = BufReader::new(input_file);

    buffered.lines().try_for_each(|line| -> Result<()> {
        let line = line.with_context(|| "An error occured while reading the file")?;
        let processed_line = if line.contains("-->") {
            process_duration(&line, offset).with_context(|| {
                format!("An error occurred while processing the line `{}`", line)
            })?
        } else {
            line
        };
        writeln!(output_file, "{}", processed_line)
            .with_context(|| "An error occurred while writing to the output")?;
        Ok(())
    })?;

    fs::rename(input_file_path, String::from(input_file_path) + ".orig")
        .with_context(|| "An error occurred while taking a backup of the original file")?;

    fs::rename(output_file_path, input_file_path).with_context(|| {
        "An error occurred while trying to replace the original file with the udpated version"
    })?;

    Ok(())
}

fn process_duration(line: &str, offset: i8) -> Result<String, CustomError> {
    let separator = " --> ";
    let res: Vec<String> = line
        .split(separator)
        .map(|item| apply_offset(item, offset))
        .collect::<Result<_, _>>()?;
    Ok(res.join(separator))
}

fn apply_offset(input: &str, offset: i8) -> Result<String, CustomError> {
    let format = "%H:%M:%S,%3f";
    let time = NaiveTime::parse_from_str(input, format)
        .map_err(|err| CustomError::ParseError {
            value: String::from(input),
            source_error: err.to_string(),
        })?
        .overflowing_add_signed(Duration::milliseconds(offset as i64))
        .0
        .format(format)
        .to_string();
    Ok(time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;
    #[test_case("01:29:13,905", 0    => Ok(String::from("01:29:13,905")) ; "zero offset")]
    #[test_case("01:29:13,905", 100  => Ok(String::from("01:29:14,005")) ; "positive offset")]
    #[test_case("01:29:13,905", -100 => Ok(String::from("01:29:13,805")) ; "negative offset")]
    #[test_case("01:29:13:905", 10   => Err(CustomError::ParseError{ value: String::from("01:29:13:905"), source_error: String::from("input contains invalid characters")}) ; "invalid input format")]
    fn test_apply_offset(input: &str, offset: i8) -> Result<String, CustomError> {
        apply_offset(input, offset)
    }

    #[test_case("01:29:08,934 --> 01:29:13,903", 100 => Ok(String::from("01:29:09,034 --> 01:29:14,003")); "works")]
    fn test_process_duration(input: &str, offset: i8) -> Result<String, CustomError> {
        process_duration(input, offset)
    }
}
