use std::fs::File;
use std::io::BufRead;
use std::io::BufWriter;
use std::io::Write;
use std::{collections::HashMap, fs::OpenOptions, path::PathBuf};
use tracing::error;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::config::app_dir;
use crate::race::StartNumber;

#[derive(Debug, Deserialize, Serialize)]
struct RacerFinish {
    start_number: StartNumber,
    finish: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TrackStart {
    track: String,
    start: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum EventType {
    #[serde(rename = "racer_finish")]
    RacerFinish(RacerFinish),
    #[serde(rename = "track_start")]
    TrackStart(TrackStart),
}

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    timestamp: DateTime<Utc>,
    event: EventType,
}

impl PartialEq for RaceEvents {
    fn eq(&self, other: &Self) -> bool {
        self.track_starts == other.track_starts
    }
}

pub struct RaceEvents {
    writer: BufWriter<File>,
    track_starts: HashMap<String, DateTime<Utc>>,
    finish_times: HashMap<StartNumber, DateTime<Utc>>,
}

impl RaceEvents {
    pub fn load(race_id: u32) -> Self {
        let path = app_dir().join(format!("race_{race_id}.jsonl"));
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .unwrap();

        let writer = BufWriter::new(file);
        let (track_starts, finish_times) = load_events(&path);

        RaceEvents {
            writer,
            track_starts,
            finish_times,
        }
    }

    pub fn log_finish(&mut self, start_number: StartNumber, finish: DateTime<Utc>) {
        let line = serde_json::to_string(&Event {
            timestamp: Utc::now(),
            event: EventType::RacerFinish(RacerFinish {
                start_number,
                finish,
            }),
        })
        .unwrap();
        writeln!(self.writer, "{line}").unwrap();
        self.writer.flush().unwrap();
    }

    pub fn log_start(&mut self, track: &str, start: DateTime<Utc>) {
        let line = serde_json::to_string(&Event {
            timestamp: Utc::now(),
            event: EventType::TrackStart(TrackStart {
                track: track.to_string(),
                start,
            }),
        })
        .unwrap();
        writeln!(self.writer, "{line}").unwrap();
        self.writer.flush().unwrap();
    }

    pub fn get_finish_time_for(&self, start_number: StartNumber) -> Option<DateTime<Utc>> {
        Some(*self.finish_times.get(&start_number)?)
    }

    pub fn get_track_start(&self, track: &str) -> Option<DateTime<Utc>> {
        Some(*self.track_starts.get(track)?)
    }
}

fn parse_event(line: Result<String, std::io::Error>) -> Result<EventType, std::io::Error> {
    let event = serde_json::from_str::<Event>(&line?)?;
    Ok(event.event)
}

fn load_events(
    path: &PathBuf,
) -> (
    HashMap<String, DateTime<Utc>>,
    HashMap<StartNumber, DateTime<Utc>>,
) {
    let mut track_starts = HashMap::new();
    let mut finish_times = HashMap::new();

    let file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            warn!("Could not open log: {err:?}");
            return (HashMap::new(), HashMap::new());
        }
    };

    for line in std::io::BufReader::new(file).lines() {
        match parse_event(line) {
            Ok(EventType::RacerFinish(RacerFinish {
                start_number,
                finish,
            })) => {
                finish_times.insert(start_number, finish);
            }
            Ok(EventType::TrackStart(TrackStart { track, start })) => {
                track_starts.insert(track, start);
            }
            Err(err) => {
                error!("Failed to load record: {err:?}")
            }
        }
    }

    (track_starts, finish_times)
}
