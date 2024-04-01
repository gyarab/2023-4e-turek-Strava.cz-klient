use bson::oid::ObjectId;
use bson::{Bson, Document, Regex};
use futures_util::stream::StreamExt;

use lazy_static::lazy_static;
use mongodb::options::{AuthMechanism, Tls, TlsOptions};
use mongodb::options::{Credential, UpdateOptions};
use mongodb::{bson::doc, options::ClientOptions, Client, Collection};

use crate::utilities::input_to_regex_string;
use std::collections::HashSet;
use std::{env, vec};
use std::path::PathBuf;
use std::time::SystemTime;
use strava_client::data_struct::{
    CantineDBEntry, DishDBEntry, OrdersCancelingSettings, Query, SettingsDBEntry, SettingsData, UserDBEntry, UserData
};
lazy_static!{
static ref ALLERGENS: HashSet<String> = HashSet::from_iter(vec!["01".to_owned(), "02".to_owned(),"03".to_owned(),"04".to_owned(),"05".to_owned(),"06".to_owned(),"07".to_owned(),"08".to_owned(),"09".to_owned(),"10".to_owned(),"11".to_owned(),"12".to_owned(),"13".to_owned(),"14".to_owned()]);
}
pub struct DbClient {
    client: mongodb::Client,
}
impl DbClient {
    pub async fn new() -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            client: self::connect().await?,
        })
    }
    pub async fn get_settings_update_time(
        &self,
        username: &str,
    ) -> Result<Option<SystemTime>, String> {
        let user = self.get_user(username).await?;
        match user {
            Some(user) => {
                let settings_update_time = user.settings_update_time;
                Ok(Some(settings_update_time))
            }
            None => Ok(None),
        }
    }
    pub async fn get_settings(&self, id: &str) -> Result<Option<OrdersCancelingSettings>, String> {
        let user = self.get_user(id).await?;
        match user {
            Some(user) => Ok(Some(user.settings)),
            None => Ok(None),
        }
    }
     async fn get_user(&self, id: &str) -> Result<Option<UserData>, String> {
        let collection = self.get_users_collection().await;
        let mut user_res = match collection
            .aggregate(
                [
                    doc! {
                        "$match": doc! {
                            "id": id
                        }
                    },
                    doc! {
                        "$unwind": doc! {
                            "path": "$settings.blacklistedDishes",
                            "preserveNullAndEmptyArrays": true

                            
                        }
                    },
                    doc! {
                        "$unwind": doc! {
                            "path": "$settings.whitelistedDishes",
                            "preserveNullAndEmptyArrays": true

                        }
                    },
                    doc! {
                        "$lookup": doc! {
                            "from": "dishes",
                            "localField": "settings.blacklistedDishes",
                            "foreignField": "_id",
                            "as": "blacklistedDish"
                        }
                    },
                    doc! {
                        "$lookup": doc! {
                            "from": "dishes",
                            "localField": "settings.whitelistedDishes",
                            "foreignField": "_id",
                            "as": "whitelistedDish"
                        }
                    },
                    doc! {
                        "$unwind": doc! {
                            "path": "$blacklistedDish",
                            "preserveNullAndEmptyArrays": true
                        }
                    },
                    doc! {
                        "$unwind": doc! {
                            "path": "$whitelistedDish",
                            "preserveNullAndEmptyArrays": true
                        }
                    },
                    doc! {
                        "$group": doc! {
                            "_id": "$_id",
                            "settingsUpdateTime": doc! {
                                "$first": "$settingsUpdateTime"
                            },
                            "blacklistedAllergens": doc! {
                                "$first": "$settings.blacklistedAllergens"
                            },
                            "strategy": doc! {
                                "$first": "$settings.strategy"
                            },
                            "blacklistedDishes": doc! {
                                "$addToSet": "$blacklistedDish"
                            },
                            "whitelistedDishes": doc! {
                                "$addToSet": "$whitelistedDish"
                            },
                            "id": doc! {
                                "$first": "$id"
                            }
                        }
                    },
                    doc! {
                        "$project": doc! {
                            "_id": "$_id",
                            "id": "$id",
                            "settings": doc! {
                                "blacklistedDishes": "$blacklistedDishes",
                                "blacklistedAllergens": "$blacklistedAllergens",
                                "whitelistedDishes": "$whitelistedDishes",
                                "strategy": "$strategy"
                            },
                            "settingsUpdateTime": "$settingsUpdateTime"
                        }
                    },
                ],
                None,
            )
            .await
        {
            Ok(stream) => stream,
            Err(e) => return Err(e.to_string()),
        };
        match user_res.next().await {
            Some(user) => match user {
                Ok(doc) => match bson::from_document::<UserData>(doc) {
                    Ok(user) => Ok(Some(user)),
                    Err(_) => Err("Failed to parse retrived from database data".to_string()),
                },
                Err(_) => Err("Error occured while retriving data from databse".to_string()),
            },
            None => Ok(None),
        }
    }
    
     async fn create_user(&self, user: UserData) -> Result<(), String> {
        let collection = self.get_users_collection().await;
        let user = UserDBEntry {
            id: user.id,
            settings: SettingsDBEntry {
                whitelisted_dishes: self.get_dishes_ids(user.settings.whitelisted_dishes).await,
                blacklisted_dishes: self.get_dishes_ids(user.settings.blacklisted_dishes).await,
                strategy: user.settings.strategy,
                blacklisted_allergens: user.settings.blacklisted_allergens,
            },
            settings_update_time: user.settings_update_time,
        };
        match collection.insert_one(user, None).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
    pub async fn update_user(&self, user: UserData) -> Result<(), String> {
        let database = self.client.database("strava");
        let collection: Collection<UserDBEntry> = database.collection("users");
        match collection
            .update_one(
                doc! { "id": user.id },
                doc! {
                        "$set": doc! { "updateTime": serde_json::to_string(&user.settings_update_time).unwrap(),
                                      "settings": serde_json::to_string(&SettingsDBEntry { 
                                           whitelisted_dishes:self.get_dishes_ids(user.settings.whitelisted_dishes).await,
                                           blacklisted_dishes:self.get_dishes_ids(user.settings.blacklisted_dishes).await,
                                           strategy: user.settings.strategy,
                                           blacklisted_allergens: user.settings.blacklisted_allergens}).unwrap()
                                        }
                },
                None,
            )
            .await{
             Ok(_) => Ok(()),
             Err(e) => Err(e.to_string())
            }
    }
    pub async fn update_user_settings(&self, user_id:&str, list: &str, action: &str, item:SettingsData) -> Result<(), String> {
        match self.get_user(user_id).await? {
            Some(_) => {
                let list = match list {
                    "blacklist"  => "blacklistedDishes",
                    "whitelist" => "whitelistedDishes",
                     "allergens" => "blacklistedAllergens",
                     "strategy" => "strategy",
                    _ => return Err("Požadovaná položka neexistuje".to_string())};
                let action_doc = match item {
                    SettingsData::Dish(item) => {
                        let id = match self.get_dish_id(&item.name, &item.allergens).await {
                            Ok(Some(id)) => id,
                            Ok(None) => { match self.insert_dish(&item).await {
                                Ok(Some(id)) => id,
                                Ok(None) => return Err("Chyba při zápisu do databáze".to_string()),
                                Err(_) => return Err("Chyba při zápisu do databáze".to_string()),
                            }},
                            Err(_) => return Err("Chyba při zápisu do databáze".to_string()),
                        };
                        create_list_update_doc(action, list, id)?
                       
                    } 
                    SettingsData::Allergen(allergen) => {
                       if !ALLERGENS.contains(&allergen) {
                           return Err("Neznámý alergen".to_string());
                       }
                        create_list_update_doc(action, list, allergen)?
                    }
                    SettingsData::Strategy(strategy) => {
                        match strategy.as_str() {
                            "cancel" | "replace" | "cancelAll"|"disabled" => {
                                doc! {
                                    "strategy": strategy
                                }
                            },
                            _ => return Err("Neznámá strategie".to_string())
                        }
                    }
                };         
                match self.get_users_collection().await.aggregate([
                    doc! {
                        "$match": doc! {
                            "id": user_id
                        }
                    },
                    doc! {
                        "$set": doc! {
                            "settings": action_doc
                        }
                    },
                    doc! {
                        "$merge": doc! {
                            "into": "users",
                            "on": "_id",
                            "whenMatched": "replace",
                            "whenNotMatched": "insert"
                        }
                    }
                ],None).await{
                    Ok(_) => Ok(()),
                    Err(e) => {println!("{}",e); Err("Chyba při zápisu do databáze".to_string())}
                }
                
            }
            None => {
                let mut balacklisted_dishes = Vec::new();
                let mut whitelisted_dishes = Vec::new();
                let mut blacklisted_allergens = Vec::new();
                let mut strategy = "disabled".to_string();
                 match item {
                    SettingsData::Dish(dish) => {                        
                        match list {
                            "blacklist" => balacklisted_dishes.push(dish),
                            "whitelist" => whitelisted_dishes.push(dish),
                            _ => return Err("Požadovaná položka neexistuje".to_string())
                        }
                        
                    },
                    SettingsData::Allergen(allergen) => {
                        if !ALLERGENS.contains(&allergen) {
                            return Err("Neznámý alergen".to_string());
                        }
                        blacklisted_allergens.push(allergen);
                    },
                    SettingsData::Strategy(item) => {
                        match strategy.as_str() {
                            "cancel" | "replace" | "cancelAll"|"disabled" => {
                                strategy = item;
                            },
                            _ => return Err("Neznámá strategie".to_string())
                        }
                    }
                }
                self.create_user(UserData {
                    id: user_id.to_string(),
                    settings: OrdersCancelingSettings {
                        whitelisted_dishes: whitelisted_dishes,
                        blacklisted_dishes: balacklisted_dishes,
                        strategy: strategy,
                        blacklisted_allergens: blacklisted_allergens,
                    },
                    settings_update_time: SystemTime::now(),
                }).await?;
              Ok(())
            }
            
        }
        
    }

    pub async fn get_cantine(
        &self,
        cantine_id: &String,
    ) -> Result<Option<CantineDBEntry>, mongodb::error::Error> {
        let collection = self.get_cantines_collection().await;
        let cantine = collection
            .find_one(doc! { "cantineId": cantine_id }, None)
            .await;
        cantine
    }
    async fn create_cantine(&self, cantine: CantineDBEntry) -> Result<(), mongodb::error::Error> {
        let collection = self.get_cantines_collection().await;
        collection.insert_one(cantine, None).await?;
        Ok(())
    }
    pub async fn update_cantine(
        &self,
        cantine_id: &str,
        cantine_history: Vec<ObjectId>,
    ) -> Result<(), mongodb::error::Error> {
        let collection = self.get_cantines_collection().await;
        let mut result = collection
            .aggregate(
                [
                    doc! {
                       "$match": doc!{
                           "cantineId": cantine_id
                        }
                    },
                    doc! {
                        "$project":  doc!{
                            "cantineId": "$cantineId",
                            "name": "$name",
                            "cantineHistory": doc!{
                                "$setUnion": [
                                    "$cantineHistory",
                                       cantine_history
                                        ]
                                    }
                        }
                    },
                ],
                None,
            )
            .await?;
        match result.next().await {
            Some(doc) => {
                let doc: CantineDBEntry = bson::from_document(doc?)?;
                collection
                    .update_one(
                        doc! { "cantineId": cantine_id },
                        doc! {
                                "$set": doc! { "cantineHistory": doc.cantine_history }
                        },
                        None,
                    )
                    .await?;
                Ok(())
            }
            None => Ok(()),
        }
    }
    pub async fn insert_cantine(
        &self,
        cantine: CantineDBEntry,
    ) -> Result<(), mongodb::error::Error> {
        match self.get_cantine(&cantine.cantine_id).await? {
            Some(_) => {
                self.update_cantine(cantine.cantine_id.as_str(), cantine.cantine_history)
                    .await?;
                Ok(())
            }
            None => {
                self.create_cantine(cantine).await?;
                Ok(())
            }
        }
    }
    pub async fn insert_dish(
        &self,
        dish: &DishDBEntry,
    ) -> Result<Option<ObjectId>, mongodb::error::Error> {
        let collection = self.get_dishes_collection().await;
        let options = UpdateOptions::builder().upsert(true).build();
        let res = collection
            .update_one(
                doc! {"name": dish.name.clone(), "allergens":dish.allergens.clone()},
                doc! { "$setOnInsert": doc!{"name":dish.name.clone(), "allergens":dish.allergens.clone()}},
                options,
            )
            .await?;
        match res.upserted_id {
            Some(id) => Ok(Some(id.as_object_id().unwrap())),
            None => Ok(None),
        }
    }
    pub async fn insert_dishes(
        &self,
        dishes: &Vec<DishDBEntry>,
    ) -> Result<Vec<ObjectId>, mongodb::error::Error> {
        let mut updated = Vec::new();
        for dish in dishes {
            match self.insert_dish(&dish).await? {
                Some(id) => updated.push(id),
                None => match self.get_dish_id(&dish.name, &dish.allergens).await {
                    Ok(Some(id)) => updated.push(id),
                    Ok(None) => continue,
                    Err(_) => continue,
                },
            }
        }
        Ok(updated)
    }
    pub async fn get_dish_id(
        &self,
        name: &String,
        allergens: &Vec<String>,
    ) -> Result<Option<ObjectId>, mongodb::error::Error> {
        let collection: Collection<Document> = self.client.database("strava").collection("dishes");
        let dish = collection
            .find_one(doc! {"name": name, "allergens": allergens}, None)
            .await?;
        match dish {
            Some(dish) => Ok(Some(dish.get_object_id("_id").unwrap().clone())),
            None => Ok(None),
        }
    }
    async fn get_users_collection(&self) -> Collection<UserDBEntry> {
        let database = self.client.database("strava");
        database.collection("users")
    }
    async fn get_cantines_collection(&self) -> Collection<CantineDBEntry> {
        let database = self.client.database("strava");
        database.collection("cantines")
    }
    async fn get_dishes_collection(&self) -> Collection<DishDBEntry> {
        let database = self.client.database("strava");
        database.collection("dishes")
    }

    pub async fn query_cantine_history(
        &self,
        cantine_id: &str,
        query: &str
    ) -> Result<Vec<DishDBEntry>, String> {
        let reslut_stream:Result<mongodb::Cursor<Document>, mongodb::error::Error> = self
            .get_cantines_collection()
            .await
            .aggregate(
                [
                    doc! {
                        "$match": doc! {
                            "cantineId": cantine_id
                        }
                    },
                    doc! {
                        "$unwind": doc! {
                            "path": "$cantineHistory",
                            "preserveNullAndEmptyArrays": false
                        }
                    },
                    doc! {
                        "$lookup": doc! {
                            "from": "dishes",
                            "localField": "cantineHistory",
                            "foreignField": "_id",
                            "as": "dish"
                        }
                    },
                    doc! {
                        "$unwind": doc! {
                            "path": "$dish",
                            "preserveNullAndEmptyArrays": false
                        }
                    },
                    doc! {
                        "$match": doc! {
                            "dish.name": doc! {
                                "$regex": Regex { pattern: input_to_regex_string(query), options: "i".to_string() }
                        }
                    }
                    },
                    doc! {
                        "$group": doc! {
                            "_id": "id",
                            "results": doc! {
                                "$push": "$dish"
                            }
                        }
                    },
                ],
                None,
            )
            .await;
        parse_result_stream_to_results(reslut_stream).await
        
    }
   
    pub async fn query_cantine_history_for_authorized_user( &self,
        cantine_id: &str,
        query: &str,list: &str, user_id:&str) -> Result<Vec<DishDBEntry>, String>{
            let list = match list {
                "blacklist" => "blacklistedDishes",
                "whitelist" => "whitelistedDishes",
                _ => return Err("Požadovaná položka neexistuje".to_string()),
            };
            let reslut_stream:Result<mongodb::Cursor<Document>, mongodb::error::Error> = self
            .get_cantines_collection()
            .await
            .aggregate(
                [
                    doc! {
                        "$match": doc! {
                            "cantineId": cantine_id
                        }
                    },
                    doc! {
                        "$lookup": doc! {
                            "from": "users",
                            "pipeline": [
                                doc! {
                                    "$match": doc! {
                                        "id": user_id
                                    }
                                }
                            ],
                            "as": "user"
                        }
                    },
                    doc! {
                        "$unwind": doc! {
                            "path": "$user",
                            "preserveNullAndEmptyArrays": true

                        }
                    },
                    doc! {
                        "$project": doc! {
                            "_id": "$_id",
                            "name": "$name",
                            "cantineId": "$cantineId",
                            "cantineHistory": doc! {
                                "$setDifference": [
                                    "$cantineHistory",
                                    format!("$user.settings.{}", list)
                                ]
                            }
                        }
                    },  
                    doc! {
                        "$unwind": doc! {
                            "path": "$cantineHistory",
                            "preserveNullAndEmptyArrays": true
                        }
                    },
                    doc! {
                        "$lookup": doc! {
                            "from": "dishes",
                            "localField": "cantineHistory",
                            "foreignField": "_id",
                            "as": "dish"
                        }
                    },
                    doc! {
                        "$unwind": doc! {
                            "path": "$dish",
                            "preserveNullAndEmptyArrays": true
                        }
                    },
                    doc! {
                        "$match": doc! {
                            "dish.name": doc! {
                                "$regex": Regex { pattern: input_to_regex_string(query), options: "i".to_string() }
                        }
                    }
                    },
                    doc! {
                        "$group": doc! {
                            "_id": "id",
                            "results": doc! {
                                "$push": "$dish"
                            }
                        }
                    },
                ],
                None,
            )
            .await;
        parse_result_stream_to_results(reslut_stream).await

    }
    pub async fn query_settings(&self, id: &str, query: &str,list_to_query: &str) -> Result<Vec<DishDBEntry>, String> { 
        let list_to_query = match list_to_query {
            "blacklist" => "blacklistedDishes",
            "whitelist" => "whitelistedDishes",
            _ => {
                return Err("Požadovaná položka neexistuje".to_string());
            }
    
        };
        let results_stream = self.get_users_collection().await.aggregate([
            doc! {
                "$match": doc! {
                    "id": id
                }
            },
            doc! {
                "$unwind": doc! {
                    "path": format!("$settings.{}", list_to_query),
                    "preserveNullAndEmptyArrays": true
                }
            },
            doc! {
                "$lookup": doc! {
                    "from": "dishes",
                    "localField": format!("settings.{}", list_to_query),
                    "foreignField": "_id",

                    "as": format!("settings.{}", list_to_query)
                }
            },
            doc! {
                "$unwind": doc! {
                    "path": format!("$settings.{}", list_to_query),
                    "preserveNullAndEmptyArrays": true
                }
            },
            doc! {
                "$match": doc! {
                  
                    format!("settings.{}.name", list_to_query): doc! {
                        "$regex": Regex { pattern: input_to_regex_string(query), options: "i".to_string() }
                    }
                }
            },
            doc! {
                "$group": doc! {
                    "_id": "_id",
                    "results": doc! {
                        "$push": format!("$settings.{}", list_to_query)
                    }
                }
            }
        ], None).await;
        parse_result_stream_to_results(results_stream).await
    }
    pub async fn get_dishes_ids(&self, dishes: Vec<DishDBEntry>) -> Vec<ObjectId> {
        let mut ids = Vec::new();
        for dish in dishes {
            match self.get_dish_id(&dish.name, &dish.allergens).await {
                Ok(Some(id)) => ids.push(id),
                Ok(None) => continue,
                Err(_) => continue,
            }
        }
        ids
    }
}

async fn connect() -> Result<mongodb::Client, mongodb::error::Error> {
    dotenv::dotenv().ok();
    let mut client_options = ClientOptions::parse(env::var("CONNECTION_STRING").unwrap()).await?;
    client_options.credential = Some(
        Credential::builder()
            .mechanism(AuthMechanism::MongoDbX509)
            .build(),
    );
    let tls_options = TlsOptions::builder()
        .cert_key_file_path(PathBuf::from(env::var("CERT_PATH").unwrap())) // for development
        //.cert_key_file_path(PathBuf::from("/usr/api/certs/cert.pem")) // for production
        .build();
    client_options.tls = Some(Tls::Enabled(tls_options));
    let client = Client::with_options(client_options)?;
    Ok(client)
}
async fn parse_result_stream_to_results<T: serde::de::DeserializeOwned>(stream: Result<mongodb::Cursor<Document>, mongodb::error::Error>) -> Result<Vec<T>, String> {
    match stream {
        Ok(mut stream) => {
            let result_option = stream.next().await;
            match result_option {
                Some(result) => match result {
                    Ok(doc) => match bson::from_document::<Query<T>>(doc) {
                        Ok(results) => Ok(results.results),
                        Err(e) => {
                            return Err(e.to_string());
                        }
                    },
                    Err(e) => {
                        return Err(e.to_string());
                    }
                },
                None => {
                    return Ok(Vec::new());
                }
            }
        }
        Err(e) => Err(e.to_string()),
    }

}
fn create_list_update_doc<T: Into<Bson>>(action: &str, list:&str, item:T) -> Result<Document,String> {
    match action {
        "add" => Ok(doc! {
            list: doc! {
                "$setUnion": [
                    format!("$settings.{}", list),  vec![item]
                ]
            }
        }),
        "remove" => Ok(doc! {
            list: doc! {
                "$setDifference": [
                    format!("$settings.{}", list),  vec![item]
                ]
            }
        }),
        _ => Err("Neznámá akce".to_string())
    }
}
