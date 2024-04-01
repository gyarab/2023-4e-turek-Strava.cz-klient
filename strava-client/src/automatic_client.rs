use crate::data_struct::{DishDBEntry, DishInfo, OrdersCancelingSettings, User};
use crate::request_builder::RequestBuilder;
use std::collections::{HashMap, HashSet};
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
pub struct AutomaticStravaClient {
    request_builder: RequestBuilder,
    settings: OrdersCancelingSettings,
}
impl AutomaticStravaClient {
    pub async fn new(
        settings: OrdersCancelingSettings,
        user: User<'_>,
    ) -> Result<AutomaticStravaClient, String> {
        let client = AutomaticStravaClient {
            request_builder: RequestBuilder::new(),
            settings: settings,
        };
        client.request_builder.do_login_request(&user).await?;
        Ok(client)
    }
    pub fn new_with_existing_request_builder(
        settings: OrdersCancelingSettings,
        request_builder: RequestBuilder,
    ) -> AutomaticStravaClient {
        AutomaticStravaClient {
            request_builder: request_builder,
            settings: settings,
        }
    }
    pub async fn cancel_orders(&self) -> Result<(), String> {
        let menu = self
            .request_builder
            .do_get_user_menu_request()
            .await
            .unwrap();
        let blacklisted_allergens: HashSet<String> =
            HashSet::from_iter(self.settings.blacklisted_allergens.clone());
        match self.settings.strategy.as_str() {
            "cancel" => {
                for (_date, dishes) in menu {
                    for (name, info) in dishes {
                        if info.order_state
                            && ((self.settings.blacklisted_dishes.contains(&DishDBEntry {
                                name: skip_none!(name.split('-').collect::<Vec<&str>>().get(1))
                                    .trim()
                                    .to_owned(),
                                allergens: info.allergens.clone(),
                            })) || (HashSet::from_iter(info.allergens)
                                .intersection(&blacklisted_allergens)
                                .count()
                                != 0))
                        {
                            println!("Canceling {}", name);
                            let _ = self
                                .request_builder
                                .do_order_dish_request(&info.id, 0)
                                .await;
                            break;
                        }
                    }
                }
            }

            "replace" => {
                let blacklisted_dishes: HashSet<DishDBEntry> =
                    HashSet::from_iter(self.settings.blacklisted_dishes.clone());
                let white_listed_dishes: HashSet<DishDBEntry> =
                    HashSet::from_iter(self.settings.whitelisted_dishes.clone());
                for (_date, dishes) in menu {
                    let mut something_to_replace = false;
                    for (name, info) in &dishes {
                        if info.order_state
                            && ((self.settings.blacklisted_dishes.contains(&DishDBEntry {
                                name: skip_none!(name.split('-').collect::<Vec<&str>>().get(1))
                                    .trim()
                                    .to_owned(),
                                allergens: info.allergens.clone(),
                            })) || (HashSet::from_iter(info.allergens.clone())
                                .intersection(&blacklisted_allergens)
                                .count()
                                != 0))
                        {
                            let _ = self
                                .request_builder
                                .do_order_dish_request(&info.id, 0)
                                .await;
                            something_to_replace = true;
                            break;
                        }
                    }
                    if something_to_replace {
                        let map: HashMap<String, DishInfo> = HashMap::from_iter(
                            dishes
                                .iter()
                                .map(|(name, info)| (name.clone(), info.clone()))
                                .collect::<Vec<(String, DishInfo)>>(),
                        );

                        let r = HashSet::from_iter(dishes.iter().map(|(name, info)| DishDBEntry {
                            name: name.clone(),
                            allergens: info.allergens.clone(),
                        }))
                        .difference(&blacklisted_dishes)
                        .cloned()
                        .collect::<Vec<DishDBEntry>>();
                        let mut res = Vec::new();
                        for dish in r {
                            if HashSet::from_iter(dish.allergens.clone())
                                .intersection(&blacklisted_allergens)
                                .count()
                                != 0
                            {
                                res.push(dish);
                            }
                        }
                        let prefered_dish = HashSet::from_iter(res.clone())
                            .intersection(&white_listed_dishes)
                            .cloned()
                            .collect::<Vec<DishDBEntry>>();
                        match prefered_dish.get(0) {
                            Some(dish) => {
                                let _ = self
                                    .request_builder
                                    .do_order_dish_request(&map.get(&dish.name).unwrap().id, 1)
                                    .await;
                            }
                            None => match res.get(0) {
                                Some(dish) => {
                                    let _ = self
                                        .request_builder
                                        .do_order_dish_request(&map.get(&dish.name).unwrap().id, 1)
                                        .await;
                                }
                                None => {
                                    continue;
                                }
                            },
                        }
                    }
                }
            }
            "cancelAll" => {
                for (_date, dishes) in menu {
                    for (_name, info) in dishes {
                        if info.order_state {
                            let _ = self
                                .request_builder
                                .do_order_dish_request(&info.id, 0)
                                .await;
                        }
                    }
                }
            }
            "disabled" => {
                return Ok(());
            }
            _ => {
                return Err("Unknown strategy".to_string());
            }
        }
        self.request_builder.do_save_orders_request().await.unwrap();
        Ok(())
    }
}
