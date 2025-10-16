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
use tracing::{error, info};

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

#[derive(Hash, Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct Track(pub String);
impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq)]
pub struct Race {
    pub id: u32,
    pub racers: Vec<Racer>,
    pub categories: Vec<Category>,
    pub tracks: Vec<Track>,
    track_starts: HashMap<Track, DateTime<Utc>>,
    log: Rc<RefCell<RaceEvents>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Racer {
    pub id: u32,
    pub start_number: StartNumber,
    pub tag: String,
    pub first_name: String,
    pub last_name: String,
    pub track: Track,
    pub track_rank: Option<u32>,
    pub categories: Vec<Category>,
    pub categories_rank: HashMap<Category, u32>,
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
    TrackRank,
    CategoriesRank,
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
            RacerField::Track => self.track.0.cmp(&other.track.0),
            RacerField::TrackRank => match (self.track_rank, other.track_rank) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            },
            RacerField::CategoriesRank => {
                // Collect union of category keys, sort them for deterministic ordering
                let a_map = &self.categories_rank;
                let b_map = &other.categories_rank;

                let mut keys: Vec<_> = a_map.keys().chain(b_map.keys()).collect();
                keys.sort();
                keys.dedup();

                for key in keys {
                    let a_rank = a_map.get(key);
                    let b_rank = b_map.get(key);

                    match (a_rank, b_rank) {
                        (Some(&a), Some(&b)) => {
                            let ord = a.cmp(&b);
                            if ord != std::cmp::Ordering::Equal {
                                return ord;
                            }
                        }
                        (Some(_), None) => return std::cmp::Ordering::Less,
                        (None, Some(_)) => return std::cmp::Ordering::Greater,
                        (None, None) => continue,
                    }
                }

                std::cmp::Ordering::Equal
            }
            RacerField::Start => self.start.cmp(&other.start),
            RacerField::Finish => self.finish.cmp(&other.finish),
            RacerField::Time => self.time.cmp(&other.time),
        }
    }
}

/// Extract all unique tracks and sort them
fn extract_tracks(api_result: &[crate::restclient::Racer]) -> Vec<Track> {
    let mut tracks: Vec<Track> = api_result
        .iter()
        .map(|racer| Track(racer.track.name.clone()))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    tracks.sort_by_key(|track| {
        track
            .0
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

                let track = Track(racer.track.name.clone());
                let start = racelog.get_track_start(&track);
                let finish =
                    start_number.and_then(|start_number| racelog.get_finish_time_for(start_number));

                Racer {
                    id: racer.id,
                    start_number: StartNumber(racer.start_number.unwrap_or(0)),
                    tag: racer.tag_id.unwrap_or("".to_string()),
                    first_name: racer.first_name,
                    last_name: racer.last_name,
                    track,
                    track_rank: None,
                    categories: racer
                        .categories
                        .into_iter()
                        .map(|category| Category(category.name))
                        .collect(),
                    categories_rank: HashMap::new(),
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
            track_starts: racelog.track_starts.clone(),
            log: RefCell::new(racelog).into(),
        };
        race.map_start_number_to_track_rank();
        race.map_start_number_to_categories_rank();
        Ok(race)
    }

    pub fn tracks_with_start(&self) -> Vec<(Track, Option<DateTime<Utc>>)> {
        self.tracks
            .iter()
            .map(|track| (track.clone(), self.track_starts.get(track).copied()))
            .collect()
    }

    pub fn start(&mut self, track: Track, time: DateTime<Utc>) {
        for racer in self.racers.iter_mut() {
            if racer.track == track {
                racer.start = Some(time);
            }
        }
        self.log.borrow_mut().log_start(&track, time);
        self.track_starts.insert(track, time);
    }

    fn finish<F>(&mut self, mut predicate: F, time: Option<DateTime<Utc>>) -> Result<(), ()>
    where
        F: for<'a> FnMut(&&'a mut Racer) -> bool,
    {
        let racer = self.racers.iter_mut().find(|r| predicate(r)).ok_or(())?;

        if racer.finish.is_some() && time.is_some() {
            error!(
                "Racer with starting number {} already has a finish time.",
                racer.start_number
            );
            return Err(());
        }

        racer.finish = time;
        racer.time = calculate_time(racer.start, racer.finish);

        if racer.finish.is_none() {
            info!("Removing finish time for racer {}", racer.start_number);
            racer.track_rank = None;
            racer.categories_rank.clear();
        }

        self.log
            .borrow_mut()
            .log_finish(racer.start_number.clone(), time);
        self.map_start_number_to_track_rank();
        self.map_start_number_to_categories_rank();
        Ok(())
    }

    pub fn finish_start_number(&mut self, start_number: StartNumber, time: Option<DateTime<Utc>>) {
        if self
            .finish(
                |r| r.start_number == start_number && r.start.is_some(),
                time,
            )
            .is_err()
        {
            error!("Racer with starting number {start_number:?} not found.");
        }
    }

    pub fn tag_finished(&mut self, tag: &str, time: Option<DateTime<Utc>>) {
        if self
            .finish(
                |r| r.tag == tag && r.start.is_some() && r.finish.is_none(),
                time,
            )
            .is_err()
        {
            error!("Racer with tag {tag} not found.");
        }
    }

    pub fn map_start_number_to_categories_rank(&mut self) {
        let categories = self.categories.clone();
        for category in categories {
            self.calculate_categories_rank(&category);
        }
    }

    fn calculate_categories_rank(&mut self, category: &Category) {
        let mut finished: Vec<&mut Racer> = self
            .racers
            .iter_mut()
            .filter(|r| r.categories.contains(category))
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

        for (index, r) in finished.into_iter().enumerate() {
            let rank: u32 = (index + 1) as u32;
            r.categories_rank.insert(category.clone(), rank);
        }
    }

    pub fn map_start_number_to_track_rank(&mut self) {
        let tracks = self.tracks.clone();
        for track in tracks {
            self.calculate_track_rank(&track);
        }
    }

    fn calculate_track_rank(&mut self, track: &Track) {
        let mut finished: Vec<&mut Racer> = self
            .racers
            .iter_mut()
            .filter(|r| r.track == *track)
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

        for (index, r) in finished.into_iter().enumerate() {
            let rank: u32 = (index + 1) as u32;
            r.track_rank = Some(rank);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::race::*;

    #[test]
    fn test_calculate_track_rank() {
        let track = Track("Track 1".to_string());

        // define finish times
        let start = Utc::now();
        let best = start + chrono::Duration::seconds(10);
        let shared = start + chrono::Duration::seconds(15);
        let best_wrong_cat = start + chrono::Duration::seconds(1);

        let racers = vec![
            // 3. place with same time as 2, but higher start number
            Racer {
                id: 2,
                start_number: StartNumber(200),
                tag: "tag2".into(),
                first_name: "Bob".into(),
                last_name: "Jones".into(),
                track: track.clone(),
                categories: vec![],
                start: Some(start),
                finish: Some(shared),
                time: Some(shared.signed_duration_since(start)),
                track_rank: None,
                categories_rank: HashMap::new(),
            },
            // 2. place
            Racer {
                id: 2,
                start_number: StartNumber(5),
                tag: "tag4".into(),
                first_name: "Liam".into(),
                last_name: "Davis".into(),
                track: track.clone(),
                categories: vec![],
                start: Some(start),
                finish: Some(shared),
                time: Some(shared.signed_duration_since(start)),
                track_rank: None,
                categories_rank: HashMap::new(),
            },
            // Did not finish
            Racer {
                id: 3,
                start_number: StartNumber(3),
                tag: "tag3".into(),
                first_name: "Charlie".into(),
                last_name: "Brown".into(),
                track: track.clone(),
                categories: vec![],
                start: Some(start),
                finish: None,
                time: None,
                track_rank: None,
                categories_rank: HashMap::new(),
            },
            // winner with best time
            Racer {
                id: 1,
                start_number: StartNumber(50),
                tag: "tag1".into(),
                first_name: "Alice".into(),
                last_name: "Smith".into(),
                track: track.clone(),
                categories: vec![],
                start: Some(start),
                finish: Some(best),
                time: Some(best.signed_duration_since(start)),
                track_rank: None,
                categories_rank: HashMap::new(),
            },
            // winner, but different category
            Racer {
                id: 3,
                start_number: StartNumber(30),
                tag: "tag3".into(),
                first_name: "John".into(),
                last_name: "Doe".into(),
                track: Track("Different track".to_string()),
                categories: vec![],
                start: Some(start),
                finish: Some(best_wrong_cat),
                time: Some(best_wrong_cat.signed_duration_since(start)),
                track_rank: None,
                categories_rank: HashMap::new(),
            },
        ];

        let mut race = Race {
            id: 1,
            racers,
            categories: vec![],
            tracks: vec![track.clone()],
            track_starts: HashMap::new(),
            log: Rc::new(RefCell::new(RaceEvents::load(100000))),
        };

        race.calculate_track_rank(&track);

        assert_eq!(Some(3), race.racers[0].track_rank);
        assert_eq!(Some(2), race.racers[1].track_rank);
        assert_eq!(None, race.racers[2].track_rank);
        assert_eq!(Some(1), race.racers[3].track_rank);
        assert_eq!(None, race.racers[4].track_rank);
    }
}
