use chrono::{DateTime, Local, TimeDelta, Utc};

pub fn format_time(datetime: Option<DateTime<Utc>>) -> String {
    match datetime {
        Some(datetime) => datetime
            .with_timezone(&Local)
            .format("%H:%M:%S%.3f")
            .to_string(),
        None => "".to_string(),
    }
}

pub fn format_time_delta(delta: Option<TimeDelta>) -> String {
    let delta = match delta {
        Some(delta) => delta,
        _ => return "".to_string(),
    };

    let total_millis = delta.num_milliseconds();
    let hours = total_millis / 1000 / 3600;
    let mins = (total_millis / 1000 / 60) % 60;
    let secs = (total_millis / 1000) % 60;
    let millis = total_millis % 1000;

    format!("{hours:02}:{mins:02}:{secs:02}.{millis:03}")
}
