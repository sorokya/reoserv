use chrono::{NaiveDateTime, TimeZone, Utc};

pub fn format_duration(other: &NaiveDateTime) -> String {
    let now = Utc::now();
    let other_dt = TimeZone::from_utc_datetime(&Utc, other);
    let duration = now.signed_duration_since(other_dt);

    if duration.num_days() >= 7 {
        let weeks = duration.num_days() / 7;

        format!(
            "{} {} ago",
            weeks,
            if weeks == 1 { "week" } else { "weeks" }
        )
    } else if duration.num_days() >= 1 {
        let days = duration.num_days();

        format!("{} {} ago", days, if days == 1 { "day" } else { "days" })
    } else if duration.num_hours() >= 1 {
        let hours = duration.num_hours();

        format!(
            "{} {} ago",
            hours,
            if hours == 1 { "hour" } else { "hours" }
        )
    } else if duration.num_minutes() >= 1 {
        let minutes = duration.num_minutes();

        format!(
            "{} {} ago",
            minutes,
            if minutes == 1 { "minute" } else { "minutes" }
        )
    } else if duration.num_seconds() >= 1 {
        let seconds = duration.num_seconds();

        format!(
            "{} {} ago",
            seconds,
            if seconds == 1 { "second" } else { "seconds" }
        )
    } else {
        "just now".to_string()
    }
}
