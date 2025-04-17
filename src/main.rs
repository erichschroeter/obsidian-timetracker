// src/main.rs

use clap::{Arg, ArgAction, Command};
use csv::Writer;
use log::{LevelFilter, debug};
use regex::Regex;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("timetracker")
        .version("1.0")
        .about("Parses Markdown journals for time tracking info")
        .arg(
            Arg::new("directory")
                .short('d')
                .long("dir")
                .help("Directory to search")
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            Arg::new("recursive")
                .short('r')
                .long("recursive")
                .help("Recurse into subdirectories")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbosity")
                .short('v')
                .long("verbosity")
                .help("Set log verbosity level (error, warn, info, debug, trace)")
                .value_parser(["error", "warn", "info", "debug", "trace"]),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output CSV file")
                .value_name("FILE"),
        )
        .get_matches();

    let level = match matches.get_one::<String>("verbosity").map(|s| s.as_str()) {
        Some("trace") => LevelFilter::Trace,
        Some("debug") => LevelFilter::Debug,
        Some("info") => LevelFilter::Info,
        Some("warn") => LevelFilter::Warn,
        Some("error") => LevelFilter::Error,
        _ => LevelFilter::Error,
    };
    env_logger::Builder::new().filter_level(level).init();

    let recursive = matches.get_flag("recursive");
    let dirs = matches.get_many::<String>("directory").unwrap();
    let output = matches.get_one::<String>("output");

    let mut entries = vec![];
    for dir in dirs {
        let path = Path::new(dir);
        if path.is_dir() {
            collect_entries(path, recursive, &mut entries)?;
        }
    }

    let mut writer: Box<dyn Write> = match output {
        Some(file) => Box::new(fs::File::create(file)?),
        None => Box::new(std::io::stdout()),
    };

    let mut csv_writer = Writer::from_writer(&mut writer);

    for entry in entries {
        debug!("parsing {}", entry.display());
        let content = fs::read_to_string(&entry)?;
        for (tag, duration) in parse_time_entries(&content, true) {
            csv_writer.write_record(&[
                tag,
                format_duration(&duration),
                entry.to_string_lossy().into_owned(),
            ])?;
        }
    }
    csv_writer.flush()?;

    Ok(())
}

fn collect_entries(
    dir: &Path,
    recursive: bool,
    entries: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new("md")) {
            entries.push(path);
        } else if recursive && path.is_dir() {
            collect_entries(&path, true, entries)?;
        }
    }
    Ok(())
}

#[derive(Default, Debug, Clone, PartialEq)]
struct TimeDuration {
    hours: u32,
    minutes: u32,
    seconds: u32,
}

fn format_duration(duration: &TimeDuration) -> String {
    let mut parts = vec![];
    if duration.hours > 0 {
        parts.push(format!("{}h", duration.hours));
    }
    if duration.minutes > 0 {
        parts.push(format!("{}m", duration.minutes));
    }
    if duration.seconds > 0 {
        parts.push(format!("{}s", duration.seconds));
    }
    parts.join("")
}

fn parse_duration(text: &str) -> TimeDuration {
    let mut duration = TimeDuration::default();
    let re = Regex::new(r"(?i)(?P<value>\d+)(?P<unit>h|m|s|d)").unwrap();
    for cap in re.captures_iter(text) {
        let value: u32 = cap["value"].parse().unwrap_or(0);
        match &cap["unit"] {
            "h" => duration.hours += value,
            "m" => duration.minutes += value,
            "s" => duration.seconds += value,
            "d" => duration.hours += value * 8,
            _ => (),
        }
    }
    duration
}

fn extract_tags(task_text: &str, current_pbi: &Option<String>, sort_tags: bool) -> String {
    let re_tags = Regex::new(r"#[a-zA-Z0-9_-]+|#pbi-\d+").unwrap();
    let mut tags: Vec<String> = re_tags
        .find_iter(task_text)
        .map(|m| m.as_str().to_string())
        .collect();

    if let Some(pbi) = current_pbi {
        if !tags.iter().any(|t| t == pbi) {
            tags.insert(0, pbi.clone());
        }
    }

    if sort_tags {
        tags.sort();
    }

    if tags.is_empty() {
        "untagged".to_string()
    } else {
        tags.join(",")
    }
}

fn parse_time_entries(content: &str, sort_tags: bool) -> Vec<(String, TimeDuration)> {
    let mut results = vec![];
    let mut current_pbi: Option<String> = None;

    let re_heading = Regex::new(r"(?i)^#+\s+Work on \[\[(\d+)\]\]").unwrap();
    let re_time_tracked =
        Regex::new(r"(?P<text>.*?)(?:\[\s*timeTracked\s*:\s*(?P<duration>[^\]]+)\])(?P<tags>.*)")
            .unwrap();

    for line in content.lines() {
        if let Some(cap) = re_heading.captures(line) {
            current_pbi = Some(format!("#pbi-{}", &cap[1]));
            continue;
        }

        if let Some(cap) = re_time_tracked.captures(line) {
            let task_text = cap.name("text").map_or("", |m| m.as_str());
            let duration_text = cap.name("duration").map_or("", |m| m.as_str());
            let tags_text = cap.name("tags").map_or("", |m| m.as_str());
            let combined_text = format!("{} {}", task_text, tags_text).trim().to_string();
            let tag_str = extract_tags(&combined_text, &current_pbi, sort_tags);
            let duration = parse_duration(duration_text);
            results.push((tag_str, duration));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_hours() {
        let duration = parse_duration("3h");
        assert_eq!(
            TimeDuration {
                hours: 3,
                minutes: 0,
                seconds: 0
            },
            duration
        );
    }

    #[test]
    fn test_parse_duration_minutes() {
        let duration = parse_duration("45m");
        assert_eq!(
            TimeDuration {
                hours: 0,
                minutes: 45,
                seconds: 0
            },
            duration
        );
    }

    #[test]
    fn test_parse_duration_seconds() {
        let duration = parse_duration("30s");
        assert_eq!(
            TimeDuration {
                hours: 0,
                minutes: 0,
                seconds: 30
            },
            duration
        );
    }

    #[test]
    fn test_parse_duration_combined_with_spaces() {
        let duration = parse_duration("2h 10m 15s");
        assert_eq!(
            TimeDuration {
                hours: 2,
                minutes: 10,
                seconds: 15
            },
            duration
        );
    }

    #[test]
    fn test_parse_duration_combined() {
        let duration = parse_duration("2h15m10s");
        assert_eq!(
            TimeDuration {
                hours: 2,
                minutes: 15,
                seconds: 10
            },
            duration
        );
    }

    #[test]
    fn test_parse_duration_days() {
        let duration = parse_duration("1d");
        assert_eq!(
            TimeDuration {
                hours: 8,
                minutes: 0,
                seconds: 0
            },
            duration
        );
    }

    #[test]
    fn test_parse_duration_invalid() {
        let duration = parse_duration("invalid");
        assert_eq!(
            TimeDuration {
                hours: 0,
                minutes: 0,
                seconds: 0
            },
            duration
        );
    }

    #[test]
    fn test_parse_duration_mixed_valid_invalid() {
        let duration = parse_duration("2h invalid 30m");
        assert_eq!(
            TimeDuration {
                hours: 2,
                minutes: 30,
                seconds: 0
            },
            duration
        );
    }

    #[test]
    fn test_extract_tags_with_pbi() {
        let task_text = "Complete task #tag1 #tag2";
        let current_pbi = Some("#pbi-123".to_string());
        let tags = extract_tags(task_text, &current_pbi, false);
        assert_eq!("#pbi-123,#tag1,#tag2", tags);
    }

    #[test]
    fn test_extract_tags_ordered() {
        let task_text = "this #c is a task #a with unordered tags #b";
        let current_pbi = None;
        let tags = extract_tags(task_text, &current_pbi, true);
        assert_eq!("#a,#b,#c", tags);
    }

    #[test]
    fn test_extract_tags_without_pbi() {
        let task_text = "Complete task #tag1 #tag2";
        let current_pbi = None;
        let tags = extract_tags(task_text, &current_pbi, false);
        assert_eq!("#tag1,#tag2", tags);
    }

    #[test]
    fn test_extract_tags_no_tags_with_pbi() {
        let task_text = "Complete task";
        let current_pbi = Some("#pbi-123".to_string());
        let tags = extract_tags(task_text, &current_pbi, false);
        assert_eq!("#pbi-123", tags);
    }

    #[test]
    fn test_extract_tags_no_tags_no_pbi() {
        let task_text = "Complete task";
        let current_pbi = None;
        let tags = extract_tags(task_text, &current_pbi, false);
        assert_eq!("untagged", tags);
    }

    #[test]
    fn test_extract_tags_sorted() {
        let task_text = "this #c is a task #a with unordered tags #b";
        let current_pbi = None;
        let tags = extract_tags(task_text, &current_pbi, true);
        assert_eq!(tags, "#a,#b,#c");
    }

    #[test]
    fn test_extract_tags_unsorted() {
        let task_text = "this #c is a task #a with unordered tags #b";
        let current_pbi = None;
        let tags = extract_tags(task_text, &current_pbi, false);
        assert_eq!(tags, "#c,#a,#b");
    }

    #[test]
    fn test_parse_time_entries_with_sorted_tags() {
        let content = r#"
        # Work on [[123]]
        - [ ] Task 1 [ timeTracked: 1h ] #c #a #b
        "#;
        let entries = parse_time_entries(content, true);
        assert_eq!("#a,#b,#c", entries[0].0);
    }

    #[test]
    fn test_parse_time_entries_with_unsorted_tags() {
        let content = r#"
        # Work on [[123]]
        - [ ] Task 1 [ timeTracked: 1h ] #c #a #b
        "#;
        let entries = parse_time_entries(content, false);
        assert_eq!("#c,#a,#b", entries[0].0);
    }

    #[test]
    fn test_parse_time_entries_with_text_before_and_after() {
        let content = r#"
        - [ ] Task 1 [ timeTracked: 1h ] more text
        "#;
        let entries = parse_time_entries(content, true);
        assert_eq!(
            TimeDuration {
                hours: 1,
                minutes: 0,
                seconds: 0
            },
            entries[0].1
        );
    }
}
