use bson::oid::ObjectId;
use chrono::prelude::*;
use serde::Deserialize;
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::SystemTime;

// structure representing date - consists of DateTime representing date and day of week in czech represented by String
#[derive(Eq, Debug, Hash, Clone)]
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

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(
            format!(
                "{} {}",
                self.date.format("%d.%m.%Y").to_string(),
                self.day_of_week.as_str()
            )
            .as_str(),
        )
    }
}
// structure representing user
#[derive(Debug, Deserialize)]
pub struct User<'a> {
    #[serde(rename = "jmeno")]
    pub username: &'a str,
    #[serde(rename = "heslo")]
    pub password: &'a str,
    #[serde(rename = "cislo")]
    pub cantine: &'a str,
    pub lang: &'a str,
    #[serde(rename = "zustatPrihlasen")]
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
        s.serialize_field("zustatPrihlasen", &self.stay_logged)?;
        s.end()
    }
}
// structure representing information about dish

#[derive(Clone, Debug, Serialize)]
pub struct DishInfo {
    pub id: String,
    pub allergens: Vec<String>,
    #[serde(rename = "orderState")]
    pub order_state: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrdersCancelingSettings {
    #[serde(rename = "blacklistedDishes")]
    pub blacklisted_dishes: Vec<DishDBEntry>,
    #[serde(rename = "whitelistedDishes")]
    pub whitelisted_dishes: Vec<DishDBEntry>,
    #[serde(rename = "blacklistedAllergens")]
    pub blacklisted_allergens: Vec<String>,
    pub strategy: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsDBEntry {
    #[serde(rename = "blacklistedDishes")]
    pub blacklisted_dishes: Vec<ObjectId>,
    #[serde(rename = "whitelistedDishes")]
    pub whitelisted_dishes: Vec<ObjectId>,
    #[serde(rename = "blacklistedAllergens")]
    pub blacklisted_allergens: Vec<String>,
    pub strategy: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserDBEntry {
    pub id: String,
    pub settings: SettingsDBEntry,
    #[serde(rename = "settingsUpdateTime")]
    pub settings_update_time: SystemTime,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserData{
    pub id: String,
    pub settings: OrdersCancelingSettings,
    #[serde(rename = "settingsUpdateTime")]
    pub settings_update_time: SystemTime,
}
#[derive(Serialize, Deserialize,Debug)]
pub struct CantineDBEntry {
    #[serde(rename = "cantineId")]
    pub cantine_id: String,
    pub name: String,
    #[serde(rename = "cantineHistory")]
    pub cantine_history: Vec<ObjectId>,
}
#[derive(Serialize, Deserialize, Debug,Clone,Eq,Hash)]
pub struct DishDBEntry {
    pub name: String,
    pub allergens: Vec<String>,
}
impl  PartialEq for DishDBEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name) && self.allergens.eq(&other.allergens)
    }
    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cantine {
    pub id: String,
    pub name: String,
}
#[derive(Serialize, Deserialize, Debug)]

pub struct CantineData {
    pub v_mesto: Vec<String>,
    pub v_ulice: Vec<String>,
    pub v_nazev: Vec<String>,
    pub zarizeni: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct SettingsRequestBody {
    pub settings: OrdersCancelingSettings,
    pub settings_update_time: SystemTime,
}
#[derive(Serialize, Deserialize)]

pub struct OrderDishRequestBody {
    pub id: String,
    pub status: bool,
}
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub settings: HashMap<String, String>,
}
#[derive(Deserialize, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub account: f64,
}
#[derive(Deserialize, Serialize)]
pub struct Query<T> {
    pub _id: String,
    pub results: Vec<T>,
}
#[derive(Deserialize, Serialize)]
pub struct DBHistoryQueryUrlString {
    pub cantine_id: String,
    pub query: String,
}
#[derive(Deserialize, Serialize)]
pub struct AuthorizedDBHistoryQueryUrlString {
    pub cantine_id: String,
    pub query: String,
    pub list: String,
}
#[derive(Deserialize, Serialize)]
pub struct SettingsQueryUrlString {
    pub query: String,
    pub list: String,
}
#[derive(Deserialize, Serialize)]
pub struct SetSettingsUrlString {
    pub action: String,
    pub list: String,
}
#[derive(Deserialize, Serialize)]
pub enum SettingsData {
    Dish(DishDBEntry),
    Allergen(String),
    Strategy(String),
}
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum RequestResult<T, R> {
    Succes(Succes<T>),
    Error(Error<R>),
    Unauthorized(Unauthorized),
}
#[derive(Deserialize, Serialize)] 
pub struct Unauthorized {
    pub _t : String,
}
#[derive(Deserialize, Serialize)]
pub struct Error<R> {
    pub error: R,
    pub _t : String,
}

#[derive(Deserialize, Serialize)]
pub struct Succes<T> {
    pub _t : String,
    pub data: T,

}

impl<R> Error<R> {
    pub fn new(error:R) -> Error<R> {
        Error {
            error : error,
            _t: "failure".to_string(),
        }
    }
}
impl Unauthorized {
    pub fn new() -> Unauthorized {
        Unauthorized {
            _t: "unauthorized".to_string(),
        }
    }
}
impl<T> Succes<T> {
    pub fn new(data: T) -> Succes<T> {
        Succes {
            _t: "success".to_string(),
            data: data,
        }
    }
}
#[derive(Deserialize, Serialize)]
pub struct SaveRequestFailiure {
    pub error: String,
    pub account: f64,
}