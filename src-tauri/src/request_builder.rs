use crate::data_struct::{Date, DishInfo,User};
use once_cell::sync::OnceCell;
use reqwest::{Client, Error, Response};
use scraper::Html;
use indexmap::IndexMap;
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
    pub async fn login(&self, user: &User<'_>) -> Result<(), String> {
        self.do_get("https://app.strava.cz/prihlasit-se?jidelna")
            .await;
        match self
            .do_post(
                "https://app.strava.cz/api/login",
                serde_json::to_string(&user).unwrap(),
            )
            .await
        {
            Ok(res) => match res.error_for_status() {
                Ok(res) => {
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
                    Ok(())
                }
                Err(e) => return Err(e.to_string()),
            },
            Err(_) => return Err("Při komunikaci se serverem došlo k chybě".to_string()),
        }
    }
    // do get request for given cantine menu page and return it
    /*
    pub fn get_cantine_menu(&self, cantinecode: &str) -> Html {
        self.do_get(
            ("https://www.strava.cz/Strava/Stravnik/Jidelnicky?zarizeni=".to_owned() + cantinecode)
                .as_str(),
        )
    }
    */
    // do get request for loqged users menu page and return it
    pub async fn get_user_menu(
        &self,
    ) -> Result<IndexMap<Date, IndexMap<String, DishInfo>>, String> {
        match self
            .do_post_template(
                "https://app.strava.cz/api/objednavky",
                r#""konto":"0","podminka":"","resetTables":"true""#.to_string(),
            )
            .await
        {
            Ok(res) => match res.error_for_status() {
                Ok(res) => {
                    let mut menu = IndexMap::new();
                    let response_json =
                        serde_json::from_str::<serde_json::Value>(&res.text().await.unwrap())
                            .unwrap();
                    let menu_json = response_json.as_object().unwrap();
                    for key in menu_json.keys() {
                        let daily_menu_json = menu_json.get(key).unwrap().as_array().unwrap();
                        let mut daily_menu = IndexMap::new();
                        for dish in daily_menu_json {
                            let dish_name = format!(
                                "{} - {}",
                                dish.get("popis").unwrap().as_str().unwrap().trim().to_string(),
                                dish.get("nazev").unwrap().as_str().unwrap().trim().to_string()
                            ).trim().to_string();
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
                                    .unwrap().to_string(),
                            ),
                            daily_menu,
                        );
                    }
                    menu.sort_keys();
                    Ok(menu)
                }
                Err(e) => return Err(e.to_string()),
            },
            Err(_) => return Err("Došlo k chybě při odesílání požadavku".to_string()),
        }
    }
   
    pub async fn do_post(&self, url: &str, body: String) -> Result<Response, Error> {
        self.client.post(url).body(body).send().await
    }
    pub async fn do_get(&self, url: &str) -> Html {
        let res = self.client.get(url).send();
        Html::parse_document(res.await.unwrap().text().await.unwrap().as_str())
    }
    pub async fn order_dish(&self, dish_id: String, amount: i8) -> Result<(), Error> {
        match self
            .do_post_template(
                "https://app.strava.cz/api/objednavky",
                format!(r#""veta":"{}","pocet":"{}"#, dish_id, amount),
            )
            .await
        {
            Ok(res) => match res.error_for_status() {
                Ok(_) => Ok(()),
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        }
    }
    pub async fn do_save_orders_request(&self) -> Result<(), Error> {
        match self
            .do_post_template(
                "https://app.strava.cz/api/saveOrders",
                r#""xml":null"#.to_string(),
            )
            .await
        {
            Ok(res) => match res.error_for_status() {
                Ok(_) => Ok(()),
                Err(e) => return Err(e),
            },
            Err(e) => return Err(e),
        }
    }
    pub async fn do_post_template(&self, url: &str, body_args: String) -> Result<Response, Error> {
        let body = format!(
            r#"{{"lang":"EN","ignoreCert":"false","sid":"{}","s5url":"{}","cislo":"{}",{}}}"#,
            self.sid.get().unwrap(),
            self.url.get().unwrap(),
            self.canteen_id.get().unwrap(),
            body_args
        );
        println!("{}", body);
        self.client.post(url).body(body).send().await
    }
}
