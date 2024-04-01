use crate::db_client::DbClient;
use crate::utilities::{filter_digits, trim_whitespace,skip_none};
use bson::oid::ObjectId;
use reqwest::Client;
use std::{collections::HashSet, error::Error};
use strava_client::data_struct::{Cantine, CantineDBEntry, CantineData, DishDBEntry};

pub struct Crawler {
    client: Client,
    db_client: DbClient,
}
impl Crawler {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            client: Client::builder().cookie_store(true).build()?,
            db_client: DbClient::new().await?,
        })
    }
    pub async fn get_cantines(&self) -> Result<Vec<Cantine>, String> {
        let res_text = match self
            .client
            .post("https:/app.strava.cz/api/zarAMesta")
            .body(r#"{"lang":"CZ"}"#)
            .header("Content-Length", 13)
            .send()
            .await
        {
            Ok(res) => match res.text().await {
                Ok(res_text) => res_text,
                Err(_) => return Err("Failed to get cantines data".to_string()),
            },

            Err(_) => return Err("Failed to get cantines data".to_string()),
        };
        let cantines_data: Vec<CantineData> = match serde_json::from_str(&res_text) {
            Ok(cantines_data) => cantines_data,
            Err(_) => return Err("Failed to parse cantines data".to_string()),
        };
        let mut cantines: Vec<Cantine> = Vec::new();
        for cantine in cantines_data {
            cantines.push(Cantine {
                id: trim_whitespace(&cantine.zarizeni.get(0).unwrap()),
                name: trim_whitespace(&format!(
                    "{}, {}, {}",
                    match cantine.v_nazev.get(0) {
                        None => continue,
                        Some(name) => match name.as_str() {
                            "" => continue,
                            _ => name,
                        }
                    },
                    skip_none!(cantine.v_mesto.get(0)),
                    skip_none!(cantine.v_ulice.get(0))
                )),
            })
        }
        Ok(cantines)
    }
    pub async fn get_cantine_menu(&self, cantine_id: &str) -> Result<Vec<DishDBEntry>, String> {
        let res_text = match self
            .client
            .post("https://app.strava.cz/api/jidelnicky")
            .body(format!(
                r#"{{"cislo": "{}", "s5url":"","lang":"CZ","ignoreCert":false }}"#,
                cantine_id
            ))
            .send()
            .await
        {
            Ok(res) => match res.error_for_status() {
                Ok(res) => match res.text().await {
                    Ok(res_text) => res_text,
                    Err(_) => return Err("Failed to get cantine menu".to_string()),
                },
                Err(_) => return Err("Failed to get cantine menu".to_string()),
            },

            Err(_) => return Err("Failed to get cantine menu".to_string()),
        };
        let cantine_menu: serde_json::Value = match serde_json::from_str(&res_text) {
            Ok(cantine_menu) => cantine_menu,
            Err(_) => return Err("Failed to parse cantine menu".to_string()),
        };
        Ok(parse_cantine_menu(cantine_menu))
    }
    pub async fn update_cantines_history(&self) -> Result<(), Box<dyn Error>> {
        let cantines = self.get_cantines().await?;
        for cantine in cantines {
            println!("Updating cantine {} {}", cantine.name, cantine.id);
            let cantine_dishes = match self.get_cantine_menu(&cantine.id).await {
                Ok(cantine_dishes) => cantine_dishes,
                Err(e) => {
                    println!("Failed to get cantine dishes {}", e);
                    continue;
                }
            };
            let mut cantine_history: HashSet<ObjectId> =
                match self.db_client.insert_dishes(&cantine_dishes).await {
                    Ok(cantine_history) => HashSet::from_iter(cantine_history),
                    Err(e) => {
                        println!("Failed to insert dishes {}", e);
                        continue;
                    }
                };
            cantine_history.extend(self.db_client.get_dishes_ids(cantine_dishes).await);
            println!("Items added {}", cantine_history.len());
            self.db_client
                .insert_cantine(CantineDBEntry {
                    cantine_id: cantine.id,
                    name: cantine.name,
                    cantine_history: cantine_history.into_iter().collect(),
                })
                .await?;
        }
        Ok(())
    }
}
fn parse_cantine_menu(cantine_menu: serde_json::Value) -> Vec<DishDBEntry> {
    let mut menu = Vec::new();
    let cantine_menu = match cantine_menu.as_array() {
        None => return menu,
        Some(cm) => match cm.get(0) {
            None => return menu,
            Some(cm) => cm,
        },
    };
    for (_day, dishes) in cantine_menu.as_object().unwrap_or(&serde_json::Map::new()) {
        for dish in skip_none!(dishes.as_array()) {
            let mut allergens = Vec::new();
            for allergen in skip_none!(skip_none!(dish.get("alergeny")).as_array()) {
                allergens.push(
                    trim_whitespace(&filter_digits(skip_none!(skip_none!(
                        skip_none!(allergen.as_array()).get(0)
                    )
                    .as_str())))
                    .to_string(),
                );
            }
            let name = trim_whitespace(&skip_none!(skip_none!(dish.get("nazev")).as_str()).to_string());
            if name == "" {
                continue;
            }
            menu.push(DishDBEntry {
                name: name,
                allergens: allergens,
            })
        }
    }
    println!("{:?}", menu.len());
    menu
}
