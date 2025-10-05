use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use std::collections::HashSet;

use crate::restclient::RaceRestAPI;

#[derive(Clone, PartialEq)]
pub struct Race {
    pub racers: Vec<Racer>,
    pub categories: Vec<String>,
    pub tracks: Vec<String>,
}

#[derive(Clone, PartialEq)]
pub struct Racer {
    pub start_number: u32,
    pub tag: String,
    pub first_name: String,
    pub last_name: String,
    pub track: String,
    pub categories: Vec<String>,
    pub start: Option<DateTime<Utc>>,
    pub finish: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RacerField {
    StartNumber,
    FirstName,
    LastName,
    TagId,
    Track,
}

impl Racer {
    pub fn cmp_by(&self, other: &Self, field: RacerField) -> Ordering {
        match field {
            RacerField::StartNumber => self.start_number.cmp(&other.start_number),
            RacerField::FirstName => self.first_name.cmp(&other.first_name),
            RacerField::LastName => self.last_name.cmp(&other.last_name),
            RacerField::TagId => self.tag.cmp(&other.tag),
            RacerField::Track => self.track.cmp(&other.track),
        }
    }
}

impl Race {
    pub async fn load(api: RaceRestAPI, race_id: u32) -> Result<Race, Box<dyn std::error::Error>> {
        let api_result = api.registrations(race_id).await?;

        let tracks = api_result
            .iter()
            .map(|racer| racer.track.name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let mut categories = HashSet::new();
        for racer in &api_result {
            for category in &racer.categories {
                categories.insert(category.name.clone());
            }
        }

        let racers = api_result
            .into_iter()
            .map(|racer| Racer {
                start_number: racer.start_number.unwrap_or(0),
                tag: racer.tag_id.unwrap_or(0).to_string(),
                first_name: racer.first_name,
                last_name: racer.last_name,
                track: racer.track.name,
                categories: racer
                    .categories
                    .into_iter()
                    .map(|category| category.name)
                    .collect(),
                start: None,
                finish: None,
            })
            .collect();

        Ok(Race {
            racers,
            categories: categories.into_iter().collect(),
            tracks,
        })
    }
}
