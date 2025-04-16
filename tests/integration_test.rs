use assert_cmd::Command;
use std::fs;

#[test]
fn test_timetracker_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journals_dir = temp_dir.path().join("Journals");
    fs::create_dir(&journals_dir).unwrap();

    let file_path = journals_dir.join("2025-01-01.md");
    fs::write(
        &file_path,
        "- [ ] #pbi-123456 I did a thing [timeTracked: 8h]",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("timetracker").unwrap();
    cmd.arg("-d").arg(journals_dir.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(format!(
            "#pbi-123456,8h,{}\n",
            file_path.to_str().unwrap()
        ));
}

#[test]
fn test_timetracker_recursive() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journals_2023_dir = temp_dir.path().join("Journals/2023");
    let journals_2024_dir = temp_dir.path().join("Journals/2024");
    fs::create_dir_all(&journals_2023_dir).unwrap();
    fs::create_dir_all(&journals_2024_dir).unwrap();

    let file_2023_path = journals_2023_dir.join("2023-01-01.md");
    let file_2024_path = journals_2024_dir.join("2024-01-01.md");

    fs::write(
        &file_2023_path,
        "- [ ] #pbi-47 I did a thing [timeTracked: 4h]",
    )
    .unwrap();

    fs::write(
        &file_2024_path,
        "- [ ] #pbi-123 hey, remember #pbi-47 [timeTracked: 1h30m]",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("timetracker").unwrap();
    cmd.arg("-r")
        .arg("-d")
        .arg(journals_2023_dir.to_str().unwrap())
        .arg("-d")
        .arg(journals_2024_dir.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(format!(
            "#pbi-47,4h,{}\n\"#pbi-123,#pbi-47\",1h30m,{}\n",
            file_2023_path.to_str().unwrap(),
            file_2024_path.to_str().unwrap()
        ));
}
