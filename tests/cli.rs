use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use predicates::prelude::*;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn when_input_file_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("subshift")?;

    cmd.arg("--file").arg("/non/existent/path");
    cmd.arg("--offset").arg("10");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read the input file"));

    Ok(())
}

#[test]
fn offsets_are_applied() -> Result<(), Box<dyn std::error::Error>> {
    struct TestFilePaths {
        original_file: PathBuf,
        backup_file: PathBuf,
    }

    impl Drop for TestFilePaths {
        fn drop(&mut self) {
            fs::remove_file(&self.original_file).expect(
                format!(
                    "could not delete the file `{}`",
                    self.original_file.to_str().unwrap()
                )
                .as_str(),
            );
            fs::remove_file(&self.backup_file).expect(
                format!(
                    "could not delete the file`{}`",
                    self.backup_file.to_str().unwrap()
                )
                .as_str(),
            );
        }
    }

    let files = TestFilePaths {
        original_file: env::temp_dir().join("test.srt"),
        backup_file: env::temp_dir().join("test.srt.orig"),
    };

    let mut file = File::create(&files.original_file)?;

    let original_file_content = r"
1
00:04:45,271 --> 00:04:48,406
When you're underwater
for months at a time,

2
00:04:48,408 --> 00:04:50,675
<i>you lose all sense
of day and night.</i>
";

    write!(file, "{}", original_file_content)?;
    file.flush()?;

    let mut cmd = Command::cargo_bin("subshift")?;
    cmd.arg("--file").arg(&files.original_file);
    cmd.arg("--offset").arg("100");

    cmd.assert().success();

    assert!(files.original_file.exists());
    assert!(files.backup_file.exists());

    let file_content_with_offset_applied = r"
1
00:04:45,371 --> 00:04:48,506
When you're underwater
for months at a time,

2
00:04:48,508 --> 00:04:50,775
<i>you lose all sense
of day and night.</i>
";

    let actual_file_content = fs::read_to_string(&files.original_file)?;
    assert_eq!(file_content_with_offset_applied, actual_file_content);

    let backup_file_content = fs::read_to_string(&files.backup_file)?;
    assert_eq!(original_file_content, backup_file_content);

    Ok(())
}
