// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use indexmap::IndexMap;
use strava_client::data_struct::{
    Date, DishDBEntry, DishInfo, Error, OrdersCancelingSettings, RequestResult,
    SaveRequestFailiure, SettingsData, Succes, Unauthorized, User, UserInfo,
};
use strava_client::strava_client::StravaClient;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;

static CLIENT: Mutex<OnceCell<StravaClient>> = Mutex::const_new(OnceCell::const_new());
#[tauri::command]
async fn login(
    username: String,
    password: String,
    cantine: String,
) -> RequestResult<UserInfo, String> {
    let u = User {
        username: &username,
        password: &password,
        cantine: &cantine,
        lang: "EN",
        stay_logged: false,
    };
    let client = CLIENT.lock().await;

    match client
        .get_or_init(|| async { StravaClient::new().await.unwrap() })
        .await
        .login(&u)
        .await
    {
        Ok(user) => {
            match client
                .get()
                .unwrap()
                .request_builder
                .do_db_auth_request(&u)
                .await
            {
                Ok(_) => {}
                Err(e) => {
                    return RequestResult::Error(Error::new(e));
                }
            }
            return RequestResult::Succes(Succes::new(user));
        }
        Err(e) => {
            if e == "Špatné uživatelské jméno nebo heslo" {
                return RequestResult::Unauthorized(Unauthorized::new());
            }
            return RequestResult::Error(Error::new(e));
        }
    }
}

#[tauri::command]
async fn get_menu_data(
) -> RequestResult<(Vec<Date>, IndexMap<Date, IndexMap<String, DishInfo>>), String> {
    match CLIENT.lock().await.get().unwrap().get_menu().await {
        Ok(menu) => {
            return RequestResult::Succes(Succes::new((menu.keys().cloned().collect(), menu)));
        }
        Err(e) => {
            return RequestResult::Error(Error::new(e));
        }
    }
}
#[tauri::command]
async fn order_dish(dish_id: String, ordered: bool) -> RequestResult<f64, String> {
    match CLIENT
        .lock()
        .await
        .get_mut()
        .unwrap()
        .order_dish(dish_id, ordered)
        .await
    {
        Ok(val) => {
            return RequestResult::Succes(Succes::new(val));
        }
        Err(e) => {
            return RequestResult::Error(Error::new(e));
        }
    }
}
#[tauri::command]
async fn save_orders() -> RequestResult<(), SaveRequestFailiure> {
    match CLIENT.lock().await.get_mut().unwrap().save_orders().await {
        Ok(_) => {
            return RequestResult::Succes(Succes::new(()));
        }
        Err(e) => {
            return RequestResult::Error(Error::new(e));
        }
    }
}
#[tauri::command]
async fn query_cantine_history(
    cantine_id: &str,
    query: &str,
    list_to_query: &str,
) -> Result<RequestResult<Vec<DishDBEntry>, String>, ()> {
    Ok(CLIENT
        .lock()
        .await
        .get()
        .unwrap()
        .query_cantine_history(cantine_id, query, list_to_query)
        .await)
}
#[tauri::command]
async fn query_settings(
    query: &str,
    list_to_query: &str,
) -> Result<RequestResult<Vec<DishDBEntry>, String>, ()> {
    Ok(CLIENT
        .lock()
        .await
        .get()
        .unwrap()
        .query_settings(query, list_to_query)
        .await)
}
#[tauri::command]
async fn fetch_settings() -> RequestResult<OrdersCancelingSettings, String> {
    CLIENT.lock().await.get().unwrap().fetch_settings().await
}
#[tauri::command]
async fn update_settings(
    settings_item: SettingsData,
    action: &str,
    list_to_update: &str,
) -> Result<RequestResult<String, String>, String> {
    Ok(CLIENT
        .lock()
        .await
        .get()
        .unwrap()
        .update_settings(settings_item, action, list_to_update)
        .await)
}
#[tauri::command]
async fn logout() -> Result<(), ()> {
    CLIENT.lock().await.get().unwrap().logout().await
}
#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_menu_data,
            login,
            order_dish,
            save_orders,
            query_cantine_history,
            query_settings,
            fetch_settings,
            update_settings,
            logout
        ])
        .run(tauri::generate_context!())
        .expect("Došlo k chybě při spouštění aplikace");
}
