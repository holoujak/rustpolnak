use chrono::TimeZone;
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono_tz::Europe::Prague;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;

const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S%.6f";

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Race {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "dateOfEvent", deserialize_with = "parse_dt")]
    pub date_of_event: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Racer {
    pub id: u32,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(rename = "startNumber")]
    pub start_number: Option<u32>,
    pub categories: Vec<Category>,
    #[serde(rename = "tagId")]
    pub tag_id: Option<String>,
    pub track: Track,
}

#[derive(Debug, Deserialize, Ord, Eq, Clone, PartialOrd, PartialEq, Hash)]
pub struct Category {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Track {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RacerResult {
    #[serde(rename = "registrationId")]
    pub registration_id: u32,
    #[serde(rename = "startTime", serialize_with = "serialize_dt")]
    pub start_time: DateTime<Utc>,
    #[serde(rename = "finishTime", serialize_with = "serialize_dt")]
    pub finish_time: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct Results {
    results: Vec<RacerResult>,
}

fn serialize_dt<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date
        .with_timezone(&Prague)
        .format(DATE_TIME_FORMAT)
        .to_string();
    serializer.serialize_str(&s)
}

fn parse_dt<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    // parse date without timezone
    let naive =
        NaiveDateTime::parse_from_str(&s, DATE_TIME_FORMAT).map_err(serde::de::Error::custom)?;

    // interpret as local time in Prague
    let local_dt = Prague
        .from_local_datetime(&naive)
        .single()
        .ok_or(serde::de::Error::custom(
            "Failed to interpret local time in Prague",
        ))?;

    // convert to UTC
    Ok(local_dt.with_timezone(&Utc))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RaceField {
    Id,
    Name,
    DateOfEvent,
}

impl Race {
    pub fn cmp_by(&self, other: &Self, field: RaceField) -> Ordering {
        match field {
            RaceField::Id => self.id.cmp(&other.id),
            RaceField::Name => self.name.cmp(&other.name),
            RaceField::DateOfEvent => self.date_of_event.cmp(&other.date_of_event),
        }
    }
}

#[derive(Clone)]
pub struct RaceRestAPI {
    client: reqwest::Client,
    url: String,
    username: String,
    password: String,
}

impl RaceRestAPI {
    pub fn new(url: &str, username: &str, password: &str) -> Self {
        RaceRestAPI {
            client: Default::default(),
            url: url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, Box<dyn std::error::Error>> {
        let resp = self
            .client
            .get(format!("{}/{}", self.url, path.trim_start_matches('/')))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn races(&self) -> Result<Vec<Race>, Box<dyn std::error::Error>> {
        self.get("/races").await
    }

    pub async fn registrations(
        &self,
        race_id: u32,
    ) -> Result<Vec<Racer>, Box<dyn std::error::Error>> {
        self.get(&format!("/races/{race_id}/registrations")).await
    }

    pub async fn results(
        &self,
        race_id: u32,
        results: Vec<RacerResult>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let body = Results { results };

        let resp = self
            .client
            .post(format!("{0}/races/{race_id}/results", self.url))
            .basic_auth(&self.username, Some(&self.password))
            .json(&body)
            .send()
            .await?;

        println!("{:#?}", resp.text().await?);

        Ok(())
    }
}
