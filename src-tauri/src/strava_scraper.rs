use crate::request_builder::RequestBuilder;
use chrono::Datelike;
use fantoccini::{Client, ClientBuilder, Locator};
use indexmap::IndexMap;
use scraper::{Html, Selector};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::time::Duration;
use std::{
    collections::HashSet,
    process::{Child, Command},
};
use url::Url;
// structure representing user
pub struct User<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub cantine: &'a str,
    pub lang: &'a str,
    pub stay_logged: bool,
}
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
// structure representing dish
pub struct Dish<'a> {
    pub name: &'a str,
    pub allergens: Vec<&'a str>,
}
#[derive(Eq, Debug, Hash, Clone)]
pub struct Date {
    pub day: i8,
    pub month: i8,
    pub day_of_week: String,
}
impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        self.day == other.day && self.month == other.month
    }
}
impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            return Some(std::cmp::Ordering::Equal);
        }
        if self.month > other.month {
            return Some(std::cmp::Ordering::Greater);
        }
        if self.month < other.month {
            return Some(std::cmp::Ordering::Less);
        }
        Some(self.day.cmp(&other.day))
    }
}
impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl Date {
    pub fn to_string(&self) -> String {
        format!("{} {}. {}.", self.day_of_week, self.day, self.month)
    }
}

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
            Err(_) => return Err("Při komunikaci se serverem došlo k chybě".to_string()),
        };
    }
    // parse given html to menu represented by following structure HashMap<date: String, HashMap<dish_name: String, (is_ordered: bool, allergens: HashSet<String>)>>
    pub async fn scraper_user_menu(
        &self,
        request_builder: &RequestBuilder,
    ) -> Result<IndexMap<String, IndexMap<String, (bool, String, Vec<String>)>>, String> {
        let api_data = request_builder.get_user_menu().await?;
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
            let date = daily_menu_html
                .select(&date_selector)
                .next()
                .unwrap()
                .inner_html();
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
                    .map(|a| a.inner_html())
                    .collect::<Vec<String>>()
                    .into_iter()
                    .collect::<String>();

                daily_menu.insert(
                    dish_name.clone(),
                    (
                        ordered,
                        api_data
                            .get(&date)
                            .unwrap()
                            .get(&dish_name)
                            .unwrap()
                            .1
                            .to_owned(),
                        allergens,
                    ),
                );
                /* output test
                let x =allergens.into_iter().collect::<String>();
                println!("{:?}", dish.select(&dishes_name_selector).into_iter().map(|a| a.inner_html()).collect::<Vec<String>>().into_iter().collect::<String>());
                println!("{}", ordered);
                println!("{}", x);¨
                */
            }
            menu.insert(date, daily_menu);
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
    // extract and return list of allergens from given dish description
    pub fn get_allergens(&self, dish_descriptin: String) -> HashSet<String> {
        let mut allergens = HashSet::new();
        // print!("{}", x);
        for c in dish_descriptin.chars().filter(|c| c.is_digit(10)) {
            if c != '0' {
                allergens.insert(c.to_string());
            }
        }
        allergens
    }
    pub async fn close(mut self) {
        self.client.close().await.unwrap();
        self.gecko.kill().unwrap();
        self.firefox.kill().unwrap();
    }
}
