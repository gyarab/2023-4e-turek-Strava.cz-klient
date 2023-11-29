use crate::request_builder::RequestBuilder;
use crate::strava_scraper::{Scraper, User};
use indexmap::IndexMap;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{env, fs};

#[derive(Deserialize, Serialize)]
struct Config {
    settings: HashMap<String, String>,
}
pub struct StravaClient {
    request_builder: RequestBuilder,
    menu: OnceCell<IndexMap<String, IndexMap<String, (bool, String, Vec<String>)>>>,
    screaper: OnceCell<Scraper>,
    settings: Config,
}
impl StravaClient {
    pub async fn new() -> Result<StravaClient, String> {
        Ok(StravaClient {
            request_builder: RequestBuilder::new(),
            menu: OnceCell::new(),
            screaper: OnceCell::new(),
            settings: match StravaClient::load_settings() {
                Ok(settings) => settings,
                Err(e) => return Err(e),
            },
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
    pub async fn get_menu(
        &self,
    ) -> Result<IndexMap<String, IndexMap<String, (bool, String, Vec<String>)>>, String> {
        match self.menu.get() {
            Some(menu) => Ok(menu.clone()),
            None => {
                let menu = match self.settings.settings.get("data_source").unwrap().as_str() {
                    "api" => self.request_builder.get_user_menu().await?,
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
    pub async fn login(&self, user: &User<'_>) -> Result<(), String> {
        match self.settings.settings.get("data_source").unwrap().as_str() {
            "api" => self.request_builder.login(&user).await?,
            "scraper" => self.screaper.get().unwrap().login(&user).await?,
            _ => {
                return Err(
                    "Chybná konfigurace způsobu získání dat v souboru config.toml".to_string(),
                )
            }
        };
        Ok(())
    }
}
