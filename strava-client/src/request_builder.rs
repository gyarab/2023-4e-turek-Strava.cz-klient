use std::{collections::HashMap, time::SystemTime};

use crate::data_struct::{
    Date, DishDBEntry, DishInfo, Error, OrdersCancelingSettings, RequestResult, SettingsData,
    Succes, Unauthorized, User, UserInfo,
};
use indexmap::IndexMap;
use once_cell::sync::{Lazy, OnceCell};
use reqwest::{Client, Response};
use scraper::Html;
use serde::Deserialize;
use serde_json::{Map, Value};
use urlencoding::encode;

static ENDPOINT: Lazy<String> = Lazy::new(|| "https://strava.jumpingcrab.com".to_string());
#[derive(Clone)]
pub struct RequestBuilder {
    client: Client,
    canteen_id: OnceCell<String>,
    sid: OnceCell<String>,
    url: OnceCell<String>,
}
impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        RequestBuilder {
            client: Client::builder().cookie_store(true).build().unwrap(),
            sid: OnceCell::new(),
            canteen_id: OnceCell::new(),
            url: OnceCell::new(),
        }
    }
    // authenticate user and retun errors if occured
    pub async fn do_login_request(&self, user: &User<'_>) -> Result<UserInfo, String> {
        self.do_get("https://app.strava.cz/prihlasit-se?jidelna")
            .await;
        match self
            .do_post(
                "https://app.strava.cz/api/login",
                serde_json::to_string(&user).unwrap(),
            )
            .await
        {
            Ok(res) => match res.status().as_u16() {
                200..=300 => {
                    let res_json =
                        serde_json::from_str::<serde_json::Value>(&res.text().await.unwrap())
                            .unwrap();
                    self.sid
                        .set(res_json.get("sid").unwrap().as_str().unwrap().to_string())
                        .unwrap();
                    self.url
                        .set(res_json.get("s5url").unwrap().as_str().unwrap().to_string())
                        .unwrap();
                    self.canteen_id.set(user.cantine.to_string()).unwrap();
                    let user = res_json.get("uzivatel").unwrap().as_object().unwrap();
                    Ok(UserInfo {
                        username: user.get("jmeno").unwrap().as_str().unwrap().to_owned(),
                        account: user
                            .get("konto")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .parse()
                            .unwrap(),
                    })
                }
                _ => match res
                    .json::<serde_json::Value>()
                    .await
                    .unwrap()
                    .get("number")
                    .unwrap()
                    .as_i64()
                    .unwrap()
                {
                    20 => return Err("Špatné uživatelské jméno nebo heslo".to_string()),
                    _ => return Err("Při komunikaci se serverem došlo k chybě".to_string()),
                },
            },
            Err(_) => return Err("Při komunikaci se serverem došlo k chybě".to_string()),
        }
    }
    // do get request for loqged users menu page and return it
    pub async fn do_get_user_menu_request(
        &self,
    ) -> Result<IndexMap<Date, IndexMap<String, DishInfo>>, String> {
        match self
            .do_post_template(
                "https://app.strava.cz/api/objednavky",
                r#""konto":"0","podminka":"","resetTables":"true""#.to_string(),
                "s5url",
            )
            .await
        {
            Ok(res) => match res.error_for_status() {
                Ok(res) => {
                    let response_json: serde_json::Value =
                        serde_json::from_str::<serde_json::Value>(&res.text().await.unwrap())
                            .unwrap();

                    Ok(self::parse_menu(response_json))
                }
                Err(e) => return Err(e.to_string()),
            },
            Err(_) => return Err("Došlo k chybě při odesílání požadavku".to_string()),
        }
    }

    pub async fn do_post(&self, url: &str, body: String) -> Result<Response, String> {
        match self.client.post(url).body(body).send().await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }
    pub async fn do_get(&self, url: &str) -> Html {
        let res = self.client.get(url).send();
        Html::parse_document(res.await.unwrap().text().await.unwrap().as_str())
    }
    pub async fn do_get_request(&self, url: &str) -> Result<Response, String> {
        match self.client.get(url).send().await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn do_order_dish_request(&self, dish_id: &String, amount: u8) -> Result<f64, String> {
        match self
            .do_post_template(
                "https://app.strava.cz/api/pridejJidloS5",
                format!(r#""veta":"{}","pocet":"{}""#, dish_id, amount),
                "url",
            )
            .await
        {
            Ok(res) => match res.status() {
                reqwest::StatusCode::OK => {
                    match serde_json::from_str::<Map<String, Value>>(&res.text().await.unwrap()) {
                        Ok(json) => match json.get("konto") {
                            Some(value) => {
                                return Ok(value.as_str().unwrap().parse().unwrap());
                            }
                            None => {
                                return Err(
                                    "Došlo k chybě serveru, zkuste to znovu později".to_string()
                                )
                            }
                        },
                        Err(_) => {
                            return Err("Došlo k chybě serveru, zkuste to znovu později".to_string())
                        }
                    }
                }
                _ => match serde_json::from_str::<Map<String, Value>>(&res.text().await.unwrap()) {
                    Ok(json) => match json.get("message") {
                        Some(value) => {
                            return Err(value.as_str().unwrap().to_string());
                        }
                        None => {
                            return Err("Došlo k chybě serveru, zkuste to znovu později".to_string())
                        }
                    },
                    Err(_) => {
                        return Err("Došlo k chybě serveru, zkuste to znovu později".to_string())
                    }
                },
            },
            Err(e) => return Err(e.to_string()),
        }
    }
    pub async fn do_save_orders_request(&self) -> Result<(), String> {
        match self
            .do_post_template(
                "https://app.strava.cz/api/saveOrders",
                r#""xml":null"#.to_string(),
                "url",
            )
            .await
        {
            Ok(res) => match res.status().as_u16() {
                200 => Ok(()),
                500..=600 => Err(serde_json::from_str::<Map<String, Value>>(
                    &res.text().await.unwrap(),
                )
                .unwrap()
                .get("message")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()),
                _ => Err("Došlo k chybě serveru, zkuste to znovu později".to_string()),
            },
            Err(e) => return Err(e),
        }
    }
    pub async fn do_post_template(
        &self,
        url: &str,
        body_args: String,
        url_arg: &str,
    ) -> Result<Response, String> {
        let body = format!(
            r#"{{"lang":"EN","ignoreCert":"false","sid":"{}","{}":"{}","cislo":"{}",{}}}"#,
            self.sid.get().unwrap(),
            url_arg,
            self.url.get().unwrap(),
            self.canteen_id.get().unwrap(),
            body_args
        );
        println!("{}", body);
        self.do_post(url, body).await
    }
    pub async fn do_db_auth_request(&self, user: &User<'_>) -> Result<UserInfo, String> {
        match self
            .do_post(
                &format!("{}/login", *ENDPOINT),
                serde_json::to_string(user).unwrap(),
            )
            .await
        {
            Ok(res) => match res.status().as_u16() {
                200 => {
                    #[derive(Deserialize)]
                    struct Response {
                        #[serde(rename = "message")]
                        _message: String,
                        user: UserInfo,
                    }
                    let res = res.json::<Response>().await.unwrap();
                    Ok(res.user)
                }
                401 => Err("Špatné uživatelské jméno nebo heslo".to_string()),
                _ => Err(
                    "Došlo k chybě serveru při načítání dat z databáze, zkuste to znovu později"
                        .to_string(),
                ),
            },
            Err(_) => Err(
                "Došlo k chybě serveru při načítání dat z databáze, zkuste to znovu později"
                    .to_string(),
            ),
        }
    }
    pub async fn do_db_logout_request(&self) -> Result<Response, String> {
        self.do_post(&format!("{}/logout", *ENDPOINT), "".to_string())
            .await
    }
    pub async fn query_cantine_history(
        &self,
        cantine_id: &str,
        query: &str,
        list_to_query: &str,
    ) -> RequestResult<Vec<DishDBEntry>, String> {
        match self
            .do_get_request(&format!(
                "{}/cantine_history?cantine_id={}&query={}&list={}",
                *ENDPOINT,
                encode(cantine_id),
                encode(query),
                encode(list_to_query)
            ))
            .await
        {
            Ok(res) => match res.status().as_u16() {
                200 => {
                    let data = res
                        .json::<HashMap<String, Vec<DishDBEntry>>>()
                        .await
                        .unwrap();

                    return RequestResult::<Vec<DishDBEntry>, String>::Succes(Succes::new(
                        data.get("result").unwrap().clone(),
                    ));
                }
                _ => RequestResult::<Vec<DishDBEntry>, String>::Error(Error::new(
                    res.json::<HashMap<String, String>>()
                        .await
                        .unwrap()
                        .get("message")
                        .unwrap()
                        .to_string(),
                )),
            },
            Err(_) => RequestResult::<Vec<DishDBEntry>, String>::Error(Error::new(
                "Došlo k chybě serveru při načítání dat z databáze, zkuste to znovu později"
                    .to_string(),
            )),
        }
    }
    pub async fn query_settings(
        &self,
        query: &str,
        list_to_query: &str,
    ) -> RequestResult<Vec<DishDBEntry>, String> {
        match self
            .do_get_request(&format!(
                "{}/settings_query?query={}&list={}",
                *ENDPOINT,
                encode(query),
                encode(list_to_query)
            ))
            .await
        {
            Ok(res) => match res.status().as_u16() {
                200 => {
                    let data = res
                        .json::<HashMap<String, Vec<DishDBEntry>>>()
                        .await
                        .unwrap();
                    return RequestResult::<Vec<DishDBEntry>, String>::Succes(Succes::new(
                        data.get("result").unwrap().clone(),
                    ));
                }
                401 => RequestResult::<Vec<DishDBEntry>, String>::Unauthorized(Unauthorized::new()),
                _ => RequestResult::<Vec<DishDBEntry>, String>::Error(Error::new(
                    res.json::<HashMap<String, String>>()
                        .await
                        .unwrap()
                        .get("message")
                        .unwrap()
                        .to_string(),
                )),
            },
            Err(_) => RequestResult::<Vec<DishDBEntry>, String>::Error(Error::new(
                "Došlo k chybě serveru při načítání dat z databáze, zkuste to znovu později"
                    .to_string(),
            )),
        }
    }
    pub async fn fetch_settings(&self) -> RequestResult<OrdersCancelingSettings, String> {
        match self
            .do_get_request(&format!("{}/user_settings", *ENDPOINT))
            .await
        {
            Ok(res) => match res.status().as_u16() {
                200 => {
                    let data = res
                        .json::<HashMap<String, OrdersCancelingSettings>>()
                        .await
                        .unwrap();
                    return RequestResult::<OrdersCancelingSettings, String>::Succes(Succes::new(
                        data.get("settings").unwrap().clone(),
                    ));
                }
                401 => RequestResult::<OrdersCancelingSettings, String>::Unauthorized(
                    Unauthorized::new(),
                ),
                304 => RequestResult::<OrdersCancelingSettings, String>::Succes(Succes::new(
                    OrdersCancelingSettings {
                        blacklisted_allergens: vec![],
                        blacklisted_dishes: vec![],
                        whitelisted_dishes: vec![],
                        strategy: "replace".to_string(),
                    },
                )),
                _ =>{ 
                    println!("{:?}", res);
                    RequestResult::<OrdersCancelingSettings, String>::Error(Error::new(
                    res.json::<HashMap<String, String>>()
                        .await
                        .unwrap()
                        .get("message")
                        .unwrap()
                        .to_string(),
                ))},
            },
            Err(_) => RequestResult::<OrdersCancelingSettings, String>::Error(Error::new(
                "Došlo k chybě serveru při načítání dat z databáze, zkuste to znovu později"
                    .to_string(),
            )),
        }
    }
    pub async fn update_settings(
        &self,
        settings_item: SettingsData,
        action: &str,
        list_to_update: &str,
    ) -> RequestResult<String, String> {
        match self
            .do_post(
                &format!(
                    "{}/user_settings?action={}&list={}",
                    *ENDPOINT,
                    encode(action),
                    encode(list_to_update)
                ),
                serde_json::to_string(&settings_item).unwrap(),
            )
            .await
        {
            Ok(res) => match res.status().as_u16() {
                200 => RequestResult::<String, String>::Succes(Succes::new("succes".to_string())),
                401 => RequestResult::<String, String>::Unauthorized(Unauthorized::new()),
                _ => RequestResult::<String, String>::Error(Error::new(
                    res.json::<HashMap<String, String>>()
                        .await
                        .unwrap()
                        .get("message")
                        .unwrap()
                        .to_string(),
                )),
            },
            Err(_) => RequestResult::<String, String>::Error(Error::new(
                "Došlo k chybě serveru při načítání dat z databáze, zkuste to znovu později"
                    .to_string(),
            )),
        }
    }

    pub async fn get_last_settings_update(&self) -> Result<SystemTime, String> {
        match self
            .do_post(
                &format!("{}/settings_update_time", *ENDPOINT),
                "".to_string(),
            )
            .await
        {
            Ok(res) => {
                return Ok(*res
                    .json::<HashMap<String, SystemTime>>()
                    .await
                    .unwrap()
                    .get("last_modified")
                    .unwrap());
            }
            Err(e) => Err(e),
        }
    }
    pub async fn get_settings(&self) -> Result<OrdersCancelingSettings, String> {
        match self
            .do_get_request(&format!("{}/user_settings", *ENDPOINT))
            .await
        {
            Ok(res) => {
                let res = res
                    .json::<HashMap<String, OrdersCancelingSettings>>()
                    .await
                    .unwrap();
                Ok(res.get("settings").unwrap().clone())
            }
            Err(e) => Err(e),
        }
    }
   
}
pub fn parse_menu(menu_string: serde_json::Value) -> IndexMap<Date, IndexMap<String, DishInfo>> {
    let mut menu = IndexMap::new();
    let menu_json = menu_string.as_object().unwrap();
    for key in menu_json.keys() {
        let daily_menu_json = menu_json.get(key).unwrap().as_array().unwrap();
        let mut daily_menu = IndexMap::new();
        for dish in daily_menu_json {
            let dish_name = format!(
                "{} - {}",
                dish.get("popis")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .trim()
                    .to_string(),
                dish.get("nazev")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .trim()
                    .to_string()
            )
            .trim()
            .to_string();
            let allergens: Vec<String> = dish
                .get("alergeny")
                .unwrap()
                .as_array()
                .unwrap()
                .into_iter()
                .map(|f| {
                    f.as_array()
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                })
                .collect();
            daily_menu.insert(
                dish_name,
                DishInfo {
                    order_state: dish.get("pocet").unwrap().as_i64().unwrap() == 1,
                    id: dish.get("veta").unwrap().as_str().unwrap().to_string(),
                    allergens: allergens,
                },
            );
        }
        menu.insert(
            Date::new(
                daily_menu_json
                    .get(0)
                    .unwrap()
                    .get("datum")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
            daily_menu,
        );
    }
    menu.sort_keys();
    menu
}
