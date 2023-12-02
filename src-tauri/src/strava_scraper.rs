use crate::data_struct::{Date, DishInfo, User};
use crate::request_builder::RequestBuilder;
use chrono::prelude::*;
use fantoccini::{Client, ClientBuilder, Locator};
use indexmap::IndexMap;
use scraper::{Html, Selector};
use std::time::Duration;
use std::{
    collections::HashSet,
    process::{Child, Command},
};
use url::Url;
pub struct Scraper {
    client: Client,
    gecko: Child,
    firefox: Child,
}
impl Scraper {
    pub async fn new() -> Scraper {
        Scraper {
            firefox: Command::new("firefox")
                .env("PATH", "./bin/firefox")
                .args(["--marionette", "--headless"])
                .spawn()
                .expect("failed to execute process"),
            gecko: Command::new("geckodriver")
                .env("PATH", "./bin")
                .args(["--marionette-port", "2828", "--connect-existing"])
                .spawn()
                .expect("UwU"),
            client: ClientBuilder::native()
                .connect("http://localhost:4444")
                .await
                .expect("failed to connect to WebDriver"),
        }
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
            Ok(x) => x.click().await.unwrap(),
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
            Err(err) => return Err(err.to_string()),
        };
    }
    // parse given html to menu represented by following structure HashMap<date: String, HashMap<dish_name: String, (is_ordered: bool, allergens: HashSet<String>)>>
    pub async fn scraper_user_menu(
        &self,
        request_builder: &RequestBuilder,
    ) -> Result<IndexMap<Date, IndexMap<String, DishInfo>>, String> {
        let api_data = request_builder.get_user_menu().await?;
        println!("{:?}", api_data);
        let page = self.get_menu_page().await;
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
                daily_menu_html
                    .select(&date_selector)
                    .next()
                    .unwrap()
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
                println!(
                    "{}, {:?}",
                    dish_name,
                    api_data.get(&date).unwrap().get(&dish_name)
                );
            }
            if !daily_menu.is_empty() {
                menu.insert(date, daily_menu);
            }
        }
        Ok(menu)
    }
    async fn get_menu_page(&self) -> Html {
        self.client
            .wait()
            .for_url(Url::parse("https://app.strava.cz/").unwrap())
            .await
            .unwrap();
        Html::parse_document(self.client.source().await.unwrap().as_str())
    }

    pub async fn close(mut self) {
        self.client.close().await.unwrap();
        self.gecko.kill().unwrap();
        self.firefox.kill().unwrap();
    }
}
