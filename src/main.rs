use anyhow::{Context, Result};
use chrono::{Duration, NaiveTime};
use clap::Parser;
use std::{
    fs,
    fs::File,
    io::{BufRead, BufReader, Write},
    ops::Add,
};

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

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    process_file(&opts.file, opts.offset)
}

fn process_file(input_file_path: &str, offset: i8) -> Result<()> {
    let input_file = File::open(input_file_path).context("Failed to read the input file")?;

    let tmp_output_file_path = String::from(input_file_path) + ".tmp";
    let mut tmp_output_file =
        File::create(&tmp_output_file_path).context("Failed to create the output file")?;

    let separator = " --> ";

    let buffered = BufReader::new(input_file);
    for line in buffered.lines() {
        let line = line.context("Failed while reading the file")?;
        let line = if line.contains(separator) {
            process_duration(&line, offset, separator)
                .with_context(|| format!("Failed to process the line `{}`", line))?
        } else {
            line
        };
        writeln!(tmp_output_file, "{}", line).context("Failed while writing to the output file")?;
    }

    fs::rename(input_file_path, String::from(input_file_path) + ".orig")
        .context("Failed while taking a backup of the input file")?;

    fs::rename(tmp_output_file_path, input_file_path)
        .context("Failed while trying to replace the original file with the updated version")?;

    Ok(())
}

fn process_duration(line: &str, offset: i8, separator: &str) -> Result<String> {
    let result: Vec<String> = line
        .split(separator)
        .map(|item| apply_offset(item, offset))
        .collect::<Result<_>>()?;
    Ok(result.join(separator))
}

fn apply_offset(input: &str, offset: i8) -> Result<String> {
    let format = "%H:%M:%S,%3f";
    let time = NaiveTime::parse_from_str(input, format)?
        .add(Duration::milliseconds(offset as i64))
        .format(format)
        .to_string();
    Ok(time)
}

#[cfg(test)]
mod tests {
    use crate::apply_offset;

    #[test]
    fn apply_offset_with_zero_offset() {
        assert_eq!(apply_offset("01:29:13,905", 0).unwrap(), "01:29:13,905");
    }

    #[test]
    fn apply_offset_with_positive_offset() {
        assert_eq!(apply_offset("01:29:13,905", 100).unwrap(), "01:29:14,005");
    }

    #[test]
    fn apply_offset_with_negative_offset() {
        assert_eq!(apply_offset("01:29:13,905", -100).unwrap(), "01:29:13,805");
    }

    #[test]
    fn apply_offset_with_invalid_format() {
        assert!(apply_offset("01:29:13:905", 10).is_err());
    }
}
