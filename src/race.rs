use chrono::{DateTime, Utc};
use chrono::{Duration, TimeDelta};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::num::ParseIntError;
use std::rc::Rc;
use std::str::FromStr;
use tracing::error;

use crate::race_events::RaceEvents;
use crate::restclient::RaceRestAPI;

#[derive(Hash, Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct StartNumber(u32);

impl fmt::Display for StartNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for StartNumber {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = s.parse()?;
        Ok(StartNumber(parsed))
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Category(pub String);

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq)]
pub struct Race {
    pub id: u32,
    pub racers: Vec<Racer>,
    pub categories: Vec<Category>,
    pub tracks: Vec<String>,
    pub tracks_rank: HashMap<String, HashMap<StartNumber, u32>>, // track -> (start_number -> rank)
    log: Rc<RefCell<RaceEvents>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Racer {
    pub id: u32,
    pub start_number: StartNumber,
    pub tag: String,
    pub first_name: String,
    pub last_name: String,
    pub track: String,
    pub categories: Vec<Category>,
    pub start: Option<DateTime<Utc>>,
    pub finish: Option<DateTime<Utc>>,
    pub time: Option<Duration>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RacerField {
    StartNumber,
    FirstName,
    LastName,
    TagId,
    Track,
    Start,
    Finish,
    Time,
}

impl Racer {
    pub fn cmp_by(&self, other: &Self, field: RacerField) -> Ordering {
        match field {
            RacerField::StartNumber => self.start_number.0.cmp(&other.start_number.0),
            RacerField::FirstName => self.first_name.cmp(&other.first_name),
            RacerField::LastName => self.last_name.cmp(&other.last_name),
            RacerField::TagId => self.tag.cmp(&other.tag),
            RacerField::Track => self.track.cmp(&other.track),
            RacerField::Start => self.start.cmp(&other.start),
            RacerField::Finish => self.finish.cmp(&other.finish),
            RacerField::Time => self.time.cmp(&other.time),
        }
    }
}

/// Extract all unique tracks and sort them
fn extract_tracks(api_result: &[crate::restclient::Racer]) -> Vec<String> {
    let mut tracks: Vec<String> = api_result
        .iter()
        .map(|racer| racer.track.name.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    tracks.sort_by_key(|track| {
        track
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
    });
    tracks
}

/// Extract all unique categories and sort them
fn extract_categories(api_result: &[crate::restclient::Racer]) -> Vec<Category> {
    let mut categories = HashSet::new();
    for racer in api_result {
        for category in &racer.categories {
            categories.insert(Category(category.name.clone()));
        }
    }
    let mut categories: Vec<Category> = categories.into_iter().collect();
    categories.sort();
    categories
}

fn calculate_time(
    start: Option<DateTime<Utc>>,
    finish: Option<DateTime<Utc>>,
) -> Option<TimeDelta> {
    match (start, finish) {
        (Some(start), Some(finish)) => Some(finish.signed_duration_since(start)),
        _ => None,
    }
}

impl Race {
    pub async fn load(api: RaceRestAPI, race_id: u32) -> Result<Race, Box<dyn std::error::Error>> {
        let api_result = api.registrations(race_id).await?;
        let racelog = RaceEvents::load(race_id);
        let tracks = extract_tracks(&api_result);
        let categories = extract_categories(&api_result);

        let racers = api_result
            .into_iter()
            .map(|racer| {
                let start_number = racer.start_number.map(StartNumber);

                let start = racelog.get_track_start(&racer.track.name);
                let finish =
                    start_number.and_then(|start_number| racelog.get_finish_time_for(start_number));

                Racer {
                    id: racer.id,
                    start_number: StartNumber(racer.start_number.unwrap_or(0)),
                    tag: racer.tag_id.unwrap_or("".to_string()),
                    first_name: racer.first_name,
                    last_name: racer.last_name,
                    track: racer.track.name.clone(),
                    categories: racer
                        .categories
                        .into_iter()
                        .map(|category| Category(category.name))
                        .collect(),
                    start,
                    finish,
                    time: calculate_time(start, finish),
                }
            })
            .collect();

        let mut race = Race {
            id: race_id,
            racers,
            categories,
            tracks,
            tracks_rank: HashMap::new(),
            log: RefCell::new(racelog).into(),
        };
        race.map_start_number_to_track_rank();
        Ok(race)
    }

    pub fn start(&mut self, track: String, time: DateTime<Utc>) {
        for racer in self.racers.iter_mut() {
            if racer.track == track {
                racer.start = Some(time);
                self.log.borrow_mut().log_start(&track, time);
            }
        }
    }

    fn finish<F>(&mut self, mut predicate: F) -> Result<(), ()>
    where
        F: for<'a> FnMut(&&'a mut Racer) -> bool,
    {
        let racer = self.racers.iter_mut().find(|r| predicate(r)).ok_or(())?;
        let finish_time = Utc::now();
        racer.finish = Some(finish_time);
        racer.time = calculate_time(racer.start, racer.finish);
        self.log
            .borrow_mut()
            .log_finish(racer.start_number.clone(), finish_time);
        self.map_start_number_to_track_rank();
        Ok(())
    }

    pub fn finish_start_number(&mut self, start_number: StartNumber) {
        if self
            .finish(|r| r.start_number == start_number && r.start.is_some() && r.finish.is_none())
            .is_err()
        {
            error!("Racer with starting number {start_number:?} not found.");
        }
    }

    pub fn tag_finished(&mut self, tag: &str) {
        if self
            .finish(|r| r.tag == tag && r.start.is_some() && r.finish.is_none())
            .is_err()
        {
            error!("Racer with tag {tag} not found.");
        }
    }

    pub fn map_start_number_to_track_rank(&mut self) {
        let tracks = self.tracks.clone();
        for track in tracks {
            self.calculate_track_rank(&track);
        }
    }

    fn calculate_track_rank(&mut self, track: &str) {
        let mut finished: Vec<&Racer> = self
            .racers
            .iter()
            .filter(|r| r.track == track)
            .filter(|r| r.finish.is_some())
            .collect();

        finished.sort_by(|a, b| {
            let ord = a.finish.cmp(&b.finish);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }

            // in case the finish times are equal, sort by start number
            a.start_number.0.cmp(&b.start_number.0)
        });

        let current_track_rank = self.tracks_rank.entry(track.to_string()).or_default();

        current_track_rank.clear(); // Clear previous rankings

        for (rank, r) in finished.into_iter().enumerate() {
            current_track_rank.insert(r.start_number.clone(), (rank + 1).try_into().unwrap());
        }
    }
}
