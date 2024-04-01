use crate::data_struct::{Config, Date, DishDBEntry, DishInfo, OrdersCancelingSettings, RequestResult, SaveRequestFailiure, SettingsData, User, UserInfo};
use crate::request_builder::RequestBuilder;
use crate::strava_scraper::Scraper;
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use std::{env, fs};

pub struct StravaClient {
    pub request_builder: RequestBuilder,
    menu: OnceCell<IndexMap<Date, IndexMap<String, DishInfo>>>,
    screaper: tokio::sync::OnceCell<Scraper>,
    account: tokio::sync::OnceCell<f64>,
    account_temp: tokio::sync::OnceCell<f64>,
    menu_buffer: tokio::sync::OnceCell<Vec<(String,bool)>>,
    settings: Config,
}
impl StravaClient {
    pub async fn new() -> Result<StravaClient, String> {
        Ok(StravaClient {
            request_builder: RequestBuilder::new(),
            menu: OnceCell::new(),
            screaper: tokio::sync::OnceCell::new(),
            account: tokio::sync::OnceCell::new(),
            account_temp: tokio::sync::OnceCell::new(),
            menu_buffer: tokio::sync::OnceCell::new(),
            settings: match StravaClient::load_settings() {
                Ok(settings) => settings,
                Err(e) => return Err(e),
            },
        })
    }
    pub async fn new_with_settings(settings: Config) -> Result<StravaClient, String> {
        Ok(StravaClient {
            request_builder: RequestBuilder::new(),
            menu: OnceCell::new(),
            screaper: tokio::sync::OnceCell::new(),
            account: tokio::sync::OnceCell::new(),
            account_temp: tokio::sync::OnceCell::new(),
            menu_buffer: tokio::sync::OnceCell::new(),
            settings: settings,
        })
    }
    fn load_settings() -> Result<Config, String> {
        match toml::from_str(
            fs::read_to_string(env::current_dir().unwrap().as_path().join("../config.toml"))
                .unwrap()
                .as_str(),
        ) {
            Ok(settings) => Ok(settings),
            Err(_) => Err("Chyba při načítání nastavení ze souboru ../settings.toml".to_string()),
        }
    }
    pub async fn get_menu(&self) -> Result<IndexMap<Date, IndexMap<String, DishInfo>>, String> {
        match self.menu.get() {
            Some(menu) => Ok(menu.clone()),
            None => {
                let menu = match self.settings.settings.get("data_source").unwrap().as_str() {
                    "api" => self.request_builder.do_get_user_menu_request().await?,
                    "scraper" => {
                        self.screaper
                            .get()
                            .unwrap()
                            .scraper_user_menu(&self.request_builder)
                            .await?
                    }
                    _ => {
                        return Err(
                            "Chybná konfigurace způsobu získání dat v souboru config.toml"
                                .to_string(),
                        )
                    }
                };
                self.menu.set(menu.clone()).unwrap();
                Ok(menu)
            }
        }
    }
    pub async fn login(&self, user: &User<'_>) -> Result<UserInfo, String> {
        match self.settings.settings.get("data_source").unwrap_or(&"api".to_owned()).as_str() {
            "api" => (),
            "scraper" => {
                self.screaper
                    .get_or_init(|| async {Scraper::new().await.unwrap()} )
                    .await
                    .login(&user)
                    .await?
            }
            _ => {
                return Err(
                    "Chybná konfigurace způsobu získání dat v souboru config.toml".to_string(),
                )
            }
        };
        let user_info = self.request_builder.do_login_request(&user).await?;
        self.account.set(user_info.account).unwrap();
        Ok(user_info)
    }
    pub async fn order_dish(&mut self, dish_id: String, ordered: bool) -> Result<f64, String> {
        let amount = if ordered { 1 } else { 0 };
        match self.account_temp.get_mut() {
            None => {
                let val = *self.account.get().unwrap()
                    + self
                        .request_builder
                        .do_order_dish_request(&dish_id, amount)
                        .await?;
                self.account_temp.set(val).unwrap();
            }
            Some(val) => {
                *val += self
                    .request_builder
                    .do_order_dish_request(&dish_id, amount)
                    .await?;
            }
        }
        match self.menu_buffer.get_mut() {
            None => {
                self.menu_buffer.set(vec![(dish_id,ordered)]).unwrap();
            }
            Some(val) => {
                val.push((dish_id,ordered));
            }
        }
        
        Ok(*self.account_temp.get().unwrap())
    }
    fn save_menu_changes(&mut self){
        let menu = self.menu.get_mut().unwrap();
        let menu_buffer = self.menu_buffer.get_mut().unwrap();
        menu_buffer.iter_mut().for_each(|dish_info| {
            menu.iter_mut().for_each(|(_date, dishes)| {
                dishes.iter_mut().for_each(|(_name, dish)| {
                    if dish.id == *dish_info.0 {
                        dish.order_state = dish_info.1;
                    }
                })
            });
        });
        menu_buffer.clear();
        *self.account.get_mut().unwrap() = *self.account_temp.get().unwrap();
    }
    pub async fn query_cantine_history(
        &self,
        cantine_id: &str,
        query: &str,
        list_to_query: &str,
    ) -> RequestResult<Vec<DishDBEntry>, String> {
        self.request_builder
            .query_cantine_history(cantine_id, query, list_to_query)
            .await
    }
    pub async fn query_settings(&self, query: &str, list_to_query: &str) -> RequestResult<Vec<DishDBEntry>, String> {
        self.request_builder.query_settings(query, list_to_query).await
    }
    pub async fn fetch_settings(&self) -> RequestResult<OrdersCancelingSettings, String> {
        self.request_builder.fetch_settings().await
    }
    pub async fn update_settings(
        &self,
        settings_item: SettingsData,
        action: &str,
        list_to_update: &str,
    ) -> RequestResult<String, String> {
        self.request_builder
            .update_settings(settings_item, action, list_to_update)
            .await
    }

    pub async fn save_orders(&mut self) -> Result<String, SaveRequestFailiure> {
       match self.request_builder.do_save_orders_request().await  {
           Ok(_) => {
             self.save_menu_changes();
             Ok("success".to_string())
           }
            Err(e) => {
                self.menu_buffer.get_mut().unwrap().clear();
               *self.account_temp.get_mut().unwrap() = *self.account.get().unwrap();
                Err(SaveRequestFailiure{error:e, account:*self.account_temp.get().unwrap()})
            }
       }     
    }
    pub async fn logout(&self) -> Result<(), ()> {
       match self.request_builder.do_db_logout_request().await {
              Ok(_) => Ok(()),
              Err(_) => Err(()),
       }
    }
}
