use chrono::{DateTime, Duration, NaiveDate, Utc};

#[derive(Debug, Clone)]
pub enum ParsedTime {
    DateTime(DateTime<Utc>),
    DateOnly(NaiveDate),
}

pub fn parse_time(input: &str) -> Result<ParsedTime, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("empty time expression".to_string());
    }
    let lower = trimmed.to_lowercase();

    // --- named keywords ---
    match lower.as_str() {
        "now" => return Ok(ParsedTime::DateTime(Utc::now())),
        "today" => {
            let today = chrono::Local::now().date_naive();
            return Ok(ParsedTime::DateOnly(today));
        }
        "tomorrow" => {
            let tomorrow = chrono::Local::now().date_naive() + Duration::days(1);
            return Ok(ParsedTime::DateOnly(tomorrow));
        }
        "yesterday" => {
            let yesterday = chrono::Local::now().date_naive() - Duration::days(1);
            return Ok(ParsedTime::DateOnly(yesterday));
        }
        "next week" => {
            let next_week = chrono::Local::now().date_naive() + Duration::weeks(1);
            return Ok(ParsedTime::DateOnly(next_week));
        }
        "last week" => {
            let last_week = chrono::Local::now().date_naive() - Duration::weeks(1);
            return Ok(ParsedTime::DateOnly(last_week));
        }
        _ => {}
    }

    // --- relative: "N unit ago" ---
    if let Some(dt) = parse_relative_past(&lower) {
        return Ok(ParsedTime::DateTime(dt));
    }

    // --- relative: "in N unit" ---
    if let Some(dt) = parse_relative_future(&lower) {
        return Ok(ParsedTime::DateTime(dt));
    }

    // --- ISO 8601 datetime with timezone (RFC 3339): "2026-01-15T10:30:00Z" ---
    if let Ok(dt) = trimmed.parse::<DateTime<Utc>>() {
        return Ok(ParsedTime::DateTime(dt));
    }

    // Try parsing RFC3339 with offset
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Ok(ParsedTime::DateTime(dt.with_timezone(&Utc)));
    }

    // --- Date-only: YYYY-MM-DD ---
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(ParsedTime::DateOnly(d));
    }

    // --- US format: MM/DD/YYYY ---
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%m/%d/%Y") {
        return Ok(ParsedTime::DateOnly(d));
    }

    Err(format!("unrecognized time expression: {:?}", input))
}

/// Parse "N unit ago" patterns.  Returns None if the string doesn't match.
fn parse_relative_past(lower: &str) -> Option<DateTime<Utc>> {
    let lower = lower.trim();
    let rest = lower.strip_suffix(" ago")?;
    let (n, unit) = split_n_unit(rest)?;
    let now = Utc::now();
    let dt = apply_offset(now, -(n as i64), unit)?;
    Some(dt)
}

/// Parse "in N unit" patterns.  Returns None if the string doesn't match.
fn parse_relative_future(lower: &str) -> Option<DateTime<Utc>> {
    let lower = lower.trim();
    let rest = lower.strip_prefix("in ")?;
    let (n, unit) = split_n_unit(rest)?;
    let now = Utc::now();
    let dt = apply_offset(now, n as i64, unit)?;
    Some(dt)
}

/// Split "3 days" → (3, "days")
fn split_n_unit(s: &str) -> Option<(i64, &str)> {
    let s = s.trim();
    let (num_str, unit) = s.split_once(' ')?;
    let n: i64 = num_str.trim().parse().ok()?;
    Some((n, unit.trim()))
}

/// Apply a signed offset in the given time unit to `base`.
fn apply_offset(base: DateTime<Utc>, amount: i64, unit: &str) -> Option<DateTime<Utc>> {
    let unit = unit.trim_end_matches('s'); // strip plural 's'
    let dt = match unit {
        "second" => base + Duration::seconds(amount),
        "minute" => base + Duration::minutes(amount),
        "hour" => base + Duration::hours(amount),
        "day" => base + Duration::days(amount),
        "week" => base + Duration::weeks(amount),
        "month" => {
            // Approximate: 30 days per month
            base + Duration::days(amount * 30)
        }
        "year" => base + Duration::days(amount * 365),
        _ => return None,
    };
    Some(dt)
}

// ─────────────────────────── tests ───────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Local, Timelike};

    fn today() -> NaiveDate {
        Local::now().date_naive()
    }

    // 1. now
    #[test]
    fn test_parse_now() {
        let before = Utc::now();
        let result = parse_time("now").unwrap();
        let after = Utc::now();
        match result {
            ParsedTime::DateTime(dt) => {
                assert!(dt >= before, "dt should be >= before");
                assert!(dt <= after, "dt should be <= after");
            }
            _ => panic!("expected DateTime variant"),
        }
    }

    // 2. today
    #[test]
    fn test_parse_today() {
        let result = parse_time("today").unwrap();
        match result {
            ParsedTime::DateOnly(d) => assert_eq!(d, today()),
            _ => panic!("expected DateOnly variant"),
        }
    }

    // 3. tomorrow
    #[test]
    fn test_parse_tomorrow() {
        let result = parse_time("tomorrow").unwrap();
        match result {
            ParsedTime::DateOnly(d) => assert_eq!(d, today() + Duration::days(1)),
            _ => panic!("expected DateOnly variant"),
        }
    }

    // 4. yesterday
    #[test]
    fn test_parse_yesterday() {
        let result = parse_time("yesterday").unwrap();
        match result {
            ParsedTime::DateOnly(d) => assert_eq!(d, today() - Duration::days(1)),
            _ => panic!("expected DateOnly variant"),
        }
    }

    // 5. next week
    #[test]
    fn test_parse_next_week() {
        let result = parse_time("next week").unwrap();
        match result {
            ParsedTime::DateOnly(d) => assert_eq!(d, today() + Duration::weeks(1)),
            _ => panic!("expected DateOnly variant"),
        }
    }

    // 6. last week
    #[test]
    fn test_parse_last_week() {
        let result = parse_time("last week").unwrap();
        match result {
            ParsedTime::DateOnly(d) => assert_eq!(d, today() - Duration::weeks(1)),
            _ => panic!("expected DateOnly variant"),
        }
    }

    // 7. N days ago
    #[test]
    fn test_parse_days_ago() {
        let before = Utc::now();
        let result = parse_time("3 days ago").unwrap();
        match result {
            ParsedTime::DateTime(dt) => {
                let expected = before - Duration::days(3);
                // Allow ±2 seconds for test execution time
                assert!(
                    (dt - expected).num_seconds().abs() <= 2,
                    "dt={dt} expected~{expected}"
                );
            }
            _ => panic!("expected DateTime variant"),
        }
    }

    // 8. N hours ago
    #[test]
    fn test_parse_hours_ago() {
        let before = Utc::now();
        let result = parse_time("2 hours ago").unwrap();
        match result {
            ParsedTime::DateTime(dt) => {
                let expected = before - Duration::hours(2);
                assert!(
                    (dt - expected).num_seconds().abs() <= 2,
                    "dt={dt} expected~{expected}"
                );
            }
            _ => panic!("expected DateTime variant"),
        }
    }

    // 9. in N days
    #[test]
    fn test_parse_in_days() {
        let before = Utc::now();
        let result = parse_time("in 5 days").unwrap();
        match result {
            ParsedTime::DateTime(dt) => {
                let expected = before + Duration::days(5);
                assert!(
                    (dt - expected).num_seconds().abs() <= 2,
                    "dt={dt} expected~{expected}"
                );
            }
            _ => panic!("expected DateTime variant"),
        }
    }

    // 10. in N hours
    #[test]
    fn test_parse_in_hours() {
        let before = Utc::now();
        let result = parse_time("in 2 hours").unwrap();
        match result {
            ParsedTime::DateTime(dt) => {
                let expected = before + Duration::hours(2);
                assert!(
                    (dt - expected).num_seconds().abs() <= 2,
                    "dt={dt} expected~{expected}"
                );
            }
            _ => panic!("expected DateTime variant"),
        }
    }

    // 11. ISO 8601 date-only
    #[test]
    fn test_parse_iso8601_date() {
        let result = parse_time("2026-01-15").unwrap();
        match result {
            ParsedTime::DateOnly(d) => {
                assert_eq!(d.year(), 2026);
                assert_eq!(d.month(), 1);
                assert_eq!(d.day(), 15);
            }
            _ => panic!("expected DateOnly variant"),
        }
    }

    // 12. ISO 8601 datetime with UTC timezone
    #[test]
    fn test_parse_iso8601_datetime() {
        let result = parse_time("2026-01-15T10:30:00Z").unwrap();
        match result {
            ParsedTime::DateTime(dt) => {
                assert_eq!(dt.year(), 2026);
                assert_eq!(dt.month(), 1);
                assert_eq!(dt.day(), 15);
                assert_eq!(dt.hour(), 10);
                assert_eq!(dt.minute(), 30);
            }
            _ => panic!("expected DateTime variant"),
        }
    }

    // 13. US format MM/DD/YYYY
    #[test]
    fn test_parse_us_format() {
        let result = parse_time("01/15/2026").unwrap();
        match result {
            ParsedTime::DateOnly(d) => {
                assert_eq!(d.year(), 2026);
                assert_eq!(d.month(), 1);
                assert_eq!(d.day(), 15);
            }
            _ => panic!("expected DateOnly variant"),
        }
    }

    // 14. empty string → error
    #[test]
    fn test_parse_empty_fails() {
        assert!(parse_time("").is_err());
        assert!(parse_time("   ").is_err());
    }

    // 15. garbage → error
    #[test]
    fn test_parse_garbage_fails() {
        assert!(parse_time("not-a-date").is_err());
        assert!(parse_time("foobar").is_err());
    }

    // 16. case-insensitive keywords
    #[test]
    fn test_parse_case_insensitive() {
        assert!(matches!(parse_time("TODAY").unwrap(), ParsedTime::DateOnly(_)));
        assert!(matches!(parse_time("Today").unwrap(), ParsedTime::DateOnly(_)));
        assert!(matches!(parse_time("NOW").unwrap(), ParsedTime::DateTime(_)));
        assert!(matches!(
            parse_time("TOMORROW").unwrap(),
            ParsedTime::DateOnly(_)
        ));
        assert!(matches!(
            parse_time("YESTERDAY").unwrap(),
            ParsedTime::DateOnly(_)
        ));
        assert!(matches!(
            parse_time("NEXT WEEK").unwrap(),
            ParsedTime::DateOnly(_)
        ));
        assert!(matches!(
            parse_time("LAST WEEK").unwrap(),
            ParsedTime::DateOnly(_)
        ));
        assert!(matches!(
            parse_time("3 DAYS AGO").unwrap(),
            ParsedTime::DateTime(_)
        ));
        assert!(matches!(
            parse_time("IN 5 DAYS").unwrap(),
            ParsedTime::DateTime(_)
        ));
    }

    // Extra: singular forms
    #[test]
    fn test_parse_singular_forms() {
        assert!(matches!(
            parse_time("1 day ago").unwrap(),
            ParsedTime::DateTime(_)
        ));
        assert!(matches!(
            parse_time("1 hour ago").unwrap(),
            ParsedTime::DateTime(_)
        ));
        assert!(matches!(
            parse_time("1 minute ago").unwrap(),
            ParsedTime::DateTime(_)
        ));
        assert!(matches!(
            parse_time("1 week ago").unwrap(),
            ParsedTime::DateTime(_)
        ));
        assert!(matches!(
            parse_time("1 month ago").unwrap(),
            ParsedTime::DateTime(_)
        ));
        assert!(matches!(
            parse_time("in 1 day").unwrap(),
            ParsedTime::DateTime(_)
        ));
        assert!(matches!(
            parse_time("in 1 hour").unwrap(),
            ParsedTime::DateTime(_)
        ));
    }
}
