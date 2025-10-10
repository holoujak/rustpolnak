use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use csv::{Reader, Writer, WriterBuilder};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{config::app_dir, race::Racer};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Record {
    start_number: u32,
    firstname: String,
    lastname: String,
    start: DateTime<Utc>,
    finish: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RecordedTime {
    pub start: DateTime<Utc>,
    pub finish: DateTime<Utc>,
}

impl PartialEq for RaceLog {
    fn eq(&self, other: &Self) -> bool {
        self.records == other.records
    }
}

pub struct RaceLog {
    writer: Writer<File>,
    records: HashMap<u32, Record>,
}

impl RaceLog {
    pub fn load(race_id: u32) -> Self {
        let path = app_dir().join(format!("race_{race_id}.csv"));

        let file_exists = std::path::Path::new(&path).exists();
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .unwrap();
        let writer = WriterBuilder::new()
            .has_headers(!file_exists)
            .from_writer(file);

        RaceLog {
            writer,
            records: load_records(&path),
        }
    }

    pub fn log(&mut self, racer: &Racer) {
        if racer.start.is_none() || racer.finish.is_none() {
            error!("Cannot log {racer:?}");
            return;
        }

        self.writer
            .serialize(Record {
                start_number: racer.start_number,
                firstname: racer.first_name.clone(),
                lastname: racer.last_name.clone(),
                start: racer.start.unwrap(),
                finish: racer.finish.unwrap(),
            })
            .unwrap();
        self.writer.flush().unwrap();
    }

    pub fn stored_time_for(&self, start_number: u32) -> Option<RecordedTime> {
        let racer = self.records.get(&start_number)?;
        Some(RecordedTime {
            start: racer.start,
            finish: racer.finish,
        })
    }
}

fn load_records(path: &PathBuf) -> HashMap<u32, Record> {
    match Reader::from_path(path) {
        Ok(mut rdr) => {
            let mut records = HashMap::new();
            for line in rdr.deserialize::<Record>() {
                match line {
                    Ok(line) => {
                        records.insert(line.start_number, line);
                    }
                    Err(err) => {
                        error!("Failed to load record: {err}");
                    }
                }
            }
            records
        }
        Err(err) => {
            error!("Failed to open race log: {err}");
            HashMap::new()
        }
    }
}
