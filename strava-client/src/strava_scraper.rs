use crate::data_struct::{Date, DishInfo, User};
use crate::request_builder::RequestBuilder;
use chrono::prelude::*;
use fantoccini::{Client, ClientBuilder, Locator};
use indexmap::IndexMap;
use scraper::{Html, Selector};
use std::process::{Child, Command};
use std::time::Duration;
use url::Url;

macro_rules! skip_none {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => {
                continue;
            }
        }
    };
}

pub struct Scraper {
    client: Client,
    gecko: Child,
    firefox: Child,
}
impl Scraper {
    pub async fn new() -> Result<Scraper, String> {
        Ok(Scraper {
            firefox: match Command::new("firefox")
                .env("PATH", "./bin/firefox")
                .args(["--marionette", "--headless"])
                .spawn()
            {
                Ok(firefox) => firefox,
                Err(_) => return Err("Nepodařilo se spustit prohlížeč".to_string()),
            },
            gecko: match Command::new("geckodriver")
                .env("PATH", "./bin")
                .args(["--marionette-port", "2828", "--connect-existing"])
                .spawn()
            {
                Ok(gecko) => gecko,
                Err(_) => return Err("Nepodařilo se spustit geckodriver".to_string()),
            },
            client: match ClientBuilder::native()
                .connect("http://localhost:4444")
                .await
            {
                Ok(client) => client,
                Err(_) => return Err("Nepodařilo se připojit k prohlížeči".to_string()),
            },
        })
    }
    pub async fn login(&self, user: &User<'_>) -> Result<(), String> {
        match self.client.goto("https://app.strava.cz/").await {
            Ok(_) => (),
            Err(_) => return Err("Při komunikaci se serverem došlo k chybě".to_string()),
        };
        let cookie_button = self
            .client
            .wait()
            .at_most(Duration::from_millis(1000))
            .for_element(Locator::Css(
                r#"button[id*="CybotCookiebotDialogBodyButtonDecline"]"#,
            ))
            .await;
        match cookie_button {
            Ok(btn) => match btn.click().await {
                Ok(_) => (),
                Err(_) => return Err("Chyba při přihlášení".to_string()),
            },
            Err(_) => (),
        };
        self.client
            .find(Locator::Css(r#"input[placeholder*="Heslo"]"#))
            .await
            .unwrap()
            .send_keys(user.password)
            .await
            .unwrap();
        self.client
            .find(Locator::Css(r#"input[placeholder*="Uživatel"]"#))
            .await
            .unwrap()
            .send_keys(user.username)
            .await
            .unwrap();
        self.client
            .find(Locator::Css(r#"input[placeholder*="Číslo"]"#))
            .await
            .unwrap()
            .send_keys(user.cantine)
            .await
            .unwrap();

        match self
            .client
            .find(Locator::Css(r#"button[type="submit"]"#))
            .await
            .unwrap()
            .click()
            .await
        {
            Ok(_) => return Ok(()),
            Err(_) => return Err("Chybné přihlašovací údaje".to_string()),
        };
    }
    // parse given html to menu represented by following structure HashMap<date: String, HashMap<dish_name: String, (is_ordered: bool, allergens: HashSet<String>)>>
    pub async fn scraper_user_menu(
        &self,
        request_builder: &RequestBuilder,
    ) -> Result<IndexMap<Date, IndexMap<String, DishInfo>>, String> {
        let api_data = request_builder.do_get_user_menu_request().await?;
        let page = self.get_menu_page().await?;
        let now = chrono::Local::now();
        let mut menu = IndexMap::new();
        let day_selector =
            Selector::parse(format!(r#"div[id*='{}']"#, now.year()).as_str()).unwrap();
        let day_selector_next_year =
            Selector::parse(format!(r#"div[id*='{}']"#, now.year() + 1).as_str()).unwrap();
        let x = page.select(&day_selector);
        let xx = page.select(&day_selector_next_year);
        let days = x.chain(xx);
        let date_selector = Selector::parse("h2 > label").unwrap();
        let dishes_selector = Selector::parse(".InputHolder").unwrap();
        let dishes_name_selector = Selector::parse("span >span>span").unwrap();
        let allergens_selector = Selector::parse("button > span").unwrap();
        let order_state_selector = Selector::parse(r#"button[id*='table'] > svg"#).unwrap();

        for day in days {
            let daily_menu_html = Html::parse_fragment(day.html().as_str());
            let dishes_of_day = daily_menu_html.select(&dishes_selector);
            let mut daily_menu = IndexMap::new();
            let date = Date::new(
                skip_none!(daily_menu_html.select(&date_selector).next())
                    .inner_html()
                    .split(" ")
                    .skip(1)
                    .collect::<String>(),
            );
            for dish in dishes_of_day {
                let mut allergens = Vec::new();
                dish.select(&allergens_selector)
                    .into_iter()
                    .map(|a| a.inner_html())
                    .filter(|a| a != "")
                    .for_each(|a| allergens.push(a));
                let ordered = match dish.select(&order_state_selector).next() {
                    Some(_) => true,
                    _ => false,
                };
                let dish_name = dish
                    .select(&dishes_name_selector)
                    .into_iter()
                    .map(|a| format!("{} ", a.inner_html().trim()))
                    .filter(|a| a != " ")
                    .collect::<String>()
                    .trim()
                    .to_string();

                match api_data.get(&date).and_then(|x| x.get(&dish_name)) {
                    Some(info) => {
                        daily_menu.insert(
                            dish_name.clone(),
                            DishInfo {
                                id: info.id.clone(),
                                allergens: allergens,
                                order_state: ordered,
                            },
                        );
                    }
                    None => continue,
                }
            }
            if !daily_menu.is_empty() {
                menu.insert(date, daily_menu);
            }
        }
        Ok(menu)
    }
    async fn get_menu_page(&self) -> Result<Html, String> {
        match self
            .client
            .wait()
            .for_url(Url::parse("https://app.strava.cz/").unwrap())
            .await
        {
            Ok(_) => (),
            Err(_) => return Err("Nepodařilo se načíst stránku".to_string()),
        }
        match self.client.source().await {
            Ok(page) => Ok(Html::parse_document(&page)),
            Err(_) => Err("Nepodařilo se načíst stránku".to_string()),
        }
    }

    pub async fn close(mut self) -> Result<(), String> {
        if self.client.close().await.is_err()
            || self.gecko.kill().is_err()
            || self.firefox.kill().is_err()
        {
            return Err("Došlo k chybě při ukončování".to_string());
        }
        Ok(())
    }
}
