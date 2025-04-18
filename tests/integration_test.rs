use assert_cmd::Command;
use dedent::dedent;
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
        .stdout(format!("#pbi-123456,8h,{}\n", file_path.to_str().unwrap()));
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

    cmd.assert().success().stdout(format!(
        "#pbi-47,4h,{}\n\"#pbi-123,#pbi-47\",1h30m,{}\n",
        file_2023_path.to_str().unwrap(),
        file_2024_path.to_str().unwrap()
    ));
}

#[test]
fn test_timetracker_single_pbi_work_isolated() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journals_dir = temp_dir.path().join("Journals");
    fs::create_dir(&journals_dir).unwrap();

    let file_path = journals_dir.join("2025-01-01.md");
    fs::write(
        &file_path,
        "# Work on [[123456]]\n- [ ] some work on Task a [timeTracked: 2h]",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("timetracker").unwrap();
    cmd.arg("-d").arg(journals_dir.to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(format!("#pbi-123456,2h,{}\n", file_path.to_str().unwrap()));
}

#[test]
fn test_timetracker_time_tracked_separate_heading() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journals_dir = temp_dir.path().join("Journals");
    fs::create_dir(&journals_dir).unwrap();

    let file_path = journals_dir.join("2025-01-01.md");
    fs::write(
        &file_path,
        dedent!(
            r#"
        # Work on [[123456]]
        - [ ] some work on Task a [timeTracked: 2h]
        
        # Unlinked work
        - [ ] some work on Task b [timeTracked: 1h30m]"#
        ),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("timetracker").unwrap();
    cmd.arg("-d").arg(journals_dir.to_str().unwrap());
    // .arg("-v").arg("debug");

    cmd.assert().success().stdout(format!(
        "#pbi-123456,2h,{}\n,1h30m,{}\n",
        file_path.to_str().unwrap(),
        file_path.to_str().unwrap()
    ));
}

#[test]
fn test_timetracker_pbi_in_multiple_sections() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journals_dir = temp_dir.path().join("Journals");
    fs::create_dir(&journals_dir).unwrap();

    let file_path = journals_dir.join("2025-01-01.md");
    fs::write(
        &file_path,
        dedent!(
            r#"
        # Work on [[123456]]
        - [ ] some work on Task a [timeTracked: 2h]
        
        # Work on [[456789]]
        - [ ] some work on Task b [timeTracked: 1h30m] #pbi-123456
        "#
        ),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("timetracker").unwrap();
    cmd.arg("-d").arg(journals_dir.to_str().unwrap());
    // .arg("-v").arg("debug");

    cmd.assert().success().stdout(format!(
        "#pbi-123456,2h,{}\n\"#pbi-123456,#pbi-456789\",1h30m,{}\n",
        file_path.to_str().unwrap(),
        file_path.to_str().unwrap()
    ));
}

#[test]
fn test_timetracker_accumulate() {
    let temp_dir = tempfile::tempdir().unwrap();
    let journals_dir = temp_dir.path().join("Journals");
    fs::create_dir(&journals_dir).unwrap();

    let file_path_1 = journals_dir.join("2025-01-01.md");
    let file_path_2 = journals_dir.join("2025-01-02.md");

    fs::write(
        &file_path_1,
        "- #pbi-123456 Task A [timeTracked: 4h]",
    )
    .unwrap();

    fs::write(
        &file_path_2,
        "- #pbi-123456 Task B [timeTracked: 3h]",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("timetracker").unwrap();
    cmd.arg("--accumulate")
        .arg("-d")
        .arg(journals_dir.to_str().unwrap());

    cmd.assert().success().stdout(format!(
        "#pbi-123456,7h,\"{},{}\"\n",
        file_path_1.to_str().unwrap(),
        file_path_2.to_str().unwrap()
    ));
}
