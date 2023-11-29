// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use dotenv::dotenv;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use strava_client::strava_client::StravaClient;
use strava_client::strava_scraper::User;
use tokio::sync::OnceCell;

static CLIENT: OnceCell<StravaClient> = OnceCell::const_new();
/*
static mut CACHE: OnceCell<
    IndexMap<String, IndexMap<String, IndexMap<String, (bool, String, Vec<String>)>>>,
> = OnceCell::new();
*/
#[tauri::command]
async fn get_menu_data() -> IndexMap<String, IndexMap<String, (bool, String, Vec<String>)>> {
    dotenv().ok();
    let username = std::env::var("STRAVA_USERNAME").unwrap();
    let password = std::env::var("PASSWORD").unwrap();
    let cantine = std::env::var("CANTINE").unwrap();
    let u = User {
        username: &username,
        password: &password,
        cantine: &cantine,
        lang: "CZ",
        stay_logged: false,
    };
    CLIENT
        .get_or_init(|| async { StravaClient::new().await.unwrap() })
        .await
        .login(&u)
        .await
        .unwrap();
    CLIENT.get().unwrap().get_menu().await.unwrap()
}
/*
#[tauri::command]
fn sort_menus_keys(keys: Vec<&str>) -> Vec<String> {
    //let mut keys_as_date: Vec<Date>  = keys.iter().map(|x| x.replace(".", "").split(" ").map(|s| s.to_owned()).collect()).map(|y: Vec<_>| Date { day:y[1].parse().unwrap(), month:y[2].parse().unwrap(), day_of_week:y[0].to_string() }).collect();
    //keys_as_date.sort();
    //keys_as_date.iter().map(|x| x.to_string()).collect()
    unsafe {
        CACHE
            .cache_get("menu")
            .unwrap()
            .keys()
            .map(|x| x.to_string())
            .collect()
    }
}
*/
pub fn get_allergens(dish_descriptin: String) -> HashSet<String> {
    let mut allergens = HashSet::new();
    // print!("{}", x);
    for c in dish_descriptin.chars().filter(|c| c.is_digit(10)) {
        if c != '0' {
            allergens.insert(c.to_string());
        }
    }
    allergens
}
#[tokio::main]
async fn main() {
    let menu = get_menu_data().await;
    menu.keys()
        .for_each(|x| println!("{}, {:?}", x, menu.get(x).unwrap().keys()));

    /*
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_menu_data, sort_menus_keys])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    */
}
