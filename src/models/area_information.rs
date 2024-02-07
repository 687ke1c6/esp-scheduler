use chrono::{DateTime, FixedOffset, Utc};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use chrono::serde::ts_seconds;
use serde_json::from_reader;

mod date_serializer {
    use chrono::{DateTime, FixedOffset};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        let result = DateTime::parse_from_rfc3339(&time).unwrap();
        Ok(result)
    }
}

impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Event", 3)?;
        state.serialize_field("note", &self.note)?;
        state.serialize_field("start", &self.start.to_rfc3339())?;
        state.serialize_field("end", &self.end.to_rfc3339())?;
        state.end()        
    }
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct Event {
    #[serde(with = "date_serializer")]
    pub start: DateTime<FixedOffset>,
    #[serde(with = "date_serializer")]
    pub end: DateTime<FixedOffset>,
    pub note: String
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct AreaInformation {
    pub events: Vec<Event>
}

#[derive(Serialize, Deserialize)]
pub struct MyStruct {
    #[serde(with = "ts_seconds")]
    pub date: DateTime<Utc>,
}

impl AreaInformation {
    pub fn new(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: AreaInformation = from_reader(json.as_bytes())?;        
        Ok(config)
    }
}