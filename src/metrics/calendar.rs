use std::path::{Path, PathBuf};

use chrono::{Local, NaiveDate};
use icalendar::{Calendar, CalendarDateTime, Component, DatePerhapsTime};

use crate::config::Config;
use crate::metrics::{apply_calendar, walk_ics_files};
use crate::snapshot::SharedSnapshot;

pub fn refresh(config: &Config, snapshot: &SharedSnapshot) {
    let today = Local::now().date_naive();
    let mut files = Vec::new();

    let configured = config.calendar_ics.trim();
    if !configured.is_empty() {
        let path = PathBuf::from(configured);
        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            walk_ics_files(&path, &mut files);
        }
    }

    if let Some(home) = dirs::home_dir() {
        walk_ics_files(&home.join("Library/Calendars"), &mut files);
        walk_ics_files(&home.join(".local/share/evolution/calendar"), &mut files);
        walk_ics_files(&home.join(".calendars"), &mut files);
        walk_ics_files(&home.join("calendars"), &mut files);
    }

    files.sort();
    files.dedup();

    let mut events = Vec::new();
    for path in files {
        collect_from_file(&path, today, &mut events);
        if events.len() >= 8 {
            break;
        }
    }
    apply_calendar(snapshot, events);
}

fn collect_from_file(path: &Path, today: NaiveDate, events: &mut Vec<String>) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(calendar) = raw.parse::<Calendar>() else {
        return;
    };
    for event in calendar.iter().filter_map(|component| match component {
        icalendar::CalendarComponent::Event(event) => Some(event),
        _ => None,
    }) {
        if !falls_on_day(event, today) {
            continue;
        }
        let summary = event
            .get_summary()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Event".to_string());
        let stamp = event
            .get_start()
            .and_then(format_stamp)
            .unwrap_or_else(|| "all day".to_string());
        let line = format!("{stamp}  {summary}");
        if !events.contains(&line) {
            events.push(line);
        }
        if events.len() >= 8 {
            return;
        }
    }
}

fn falls_on_day(event: &icalendar::Event, today: NaiveDate) -> bool {
    let Some(start) = event.get_start().and_then(as_date) else {
        return false;
    };
    let end = event.get_end().and_then(as_date).unwrap_or(start);
    start <= today && today <= end
}

fn as_date(value: DatePerhapsTime) -> Option<NaiveDate> {
    match value {
        DatePerhapsTime::Date(date) => Some(date),
        DatePerhapsTime::DateTime(CalendarDateTime::Utc(dt)) => {
            Some(dt.with_timezone(&Local).date_naive())
        }
        DatePerhapsTime::DateTime(CalendarDateTime::Floating(dt)) => Some(dt.date()),
        DatePerhapsTime::DateTime(CalendarDateTime::WithTimezone { date_time, .. }) => {
            Some(date_time.date())
        }
    }
}

fn format_stamp(value: DatePerhapsTime) -> Option<String> {
    match value {
        DatePerhapsTime::Date(_) => Some("all day".to_string()),
        DatePerhapsTime::DateTime(CalendarDateTime::Utc(dt)) => {
            Some(dt.with_timezone(&Local).format("%H:%M").to_string())
        }
        DatePerhapsTime::DateTime(CalendarDateTime::Floating(dt)) => {
            Some(dt.format("%H:%M").to_string())
        }
        DatePerhapsTime::DateTime(CalendarDateTime::WithTimezone { date_time, .. }) => {
            Some(date_time.format("%H:%M").to_string())
        }
    }
}
