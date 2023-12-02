use chrono::prelude::*;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::Debug;

// structure representing date - consists of DateTime representing date and day of week in czech represented by String
#[derive(Eq, Debug, Hash, Clone, Serialize)]
pub struct Date {
    pub date: chrono::DateTime<Utc>,
    pub day_of_week: String,
}
impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        self.date.eq(&other.date)
    }
}
impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.date.partial_cmp(&other.date)
    }
}
impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl Date {
    // create new date from string in format dd.mm.yyyy
    pub fn new(date_string: String) -> Date {
        let date_data: Vec<u32> = date_string.split(".").map(|x| x.parse().unwrap()).collect();
        let days_of_week = HashMap::from([
            ("Sun", "Neděle"),
            ("Mon", "Pondělí"),
            ("Tue", "Úterý"),
            ("Wed", "Středa"),
            ("Thu", "Čtvrtek"),
            ("Fri", "Pátek"),
            ("Sat", "Sobota"),
        ]);
        let date = Utc
            .with_ymd_and_hms(date_data[2] as i32, date_data[1], date_data[0], 0, 0, 0)
            .unwrap();
        Date {
            date: date,
            day_of_week: days_of_week
                .get(date.weekday().to_string().as_str())
                .unwrap()
                .to_string(),
        }
    }
}
// structure representing user
pub struct User<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub cantine: &'a str,
    pub lang: &'a str,
    pub stay_logged: bool,
}
// serialize user to json in format suitable for strava api request body
impl Serialize for User<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("User", 5)?;
        s.serialize_field("heslo", &self.password)?;
        s.serialize_field("jmeno", &self.username)?;
        s.serialize_field("cislo", &self.cantine)?;
        s.serialize_field("lang", &self.lang)?;
        s.serialize_field("zustatPrihlasen", &self.stay_logged.to_string())?;
        s.end()
    }
}
// structure representing information about dish
#[derive(Clone, Debug, Serialize)]
pub struct DishInfo {
    pub id: String,
    pub allergens: Vec<String>,
    pub order_state: bool,
}
