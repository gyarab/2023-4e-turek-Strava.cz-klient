use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, Session, SessionExt, SessionMiddleware};
use actix_web::web::Query;
use actix_web::{
    cookie::Key,
    guard::{fn_guard, Any, Get, GuardContext, Not, Post},
    http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    web::Data,
    web::{get, post, resource, route},
    App, HttpResponse, HttpServer, Responder,
};
use rand::Rng;
use std::collections::HashMap;
use std::env;

use crate::crawler::Crawler;
use db_client::DbClient;
use std::sync::Mutex;
use strava_client::data_struct::{
    AuthorizedDBHistoryQueryUrlString, Config, DBHistoryQueryUrlString, DishDBEntry, OrderDishRequestBody, SetSettingsUrlString, SettingsData, SettingsQueryUrlString, User
};
use strava_client::strava_client::StravaClient;
use tokio_cron_scheduler::{Job, JobScheduler};
mod crawler;
mod db_client;
mod utilities;

struct AppState {
    db_client: DbClient,
    strava_clients: HashMap<String, StravaClient>,
}
#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let sched = JobScheduler::new().await.expect("Failed to create scheduler");
    sched.add(
        Job::new_async("0 0 1 * * *", |_uuid,  _l| {
            Box::pin(async move {
                println!("Starting cantine history update...");
                Crawler::new()
                    .await
                    .unwrap()
                    .update_cantines_history()
                    .await
                    .unwrap();
                println!("Cantine history update finished");
            })
        }).expect("Failed to create job")
    ).await.expect("Failed to add job to scheduler");
    sched.start().await.expect("Failed to start scheduler");

    dotenv::dotenv().ok();
    let state = Data::new(Mutex::new(AppState {
        db_client: DbClient::new().await.unwrap(),
        strava_clients: HashMap::new(),
    }));
    let secret_key = Key::generate();
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_http_only(true)
                    .cookie_same_site(actix_web::cookie::SameSite::None)
                    .cookie_secure(true)
                    .build(),
            )
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![AUTHORIZATION, ACCEPT])
                    .allowed_header(CONTENT_TYPE)
                    .supports_credentials(),
            )
            .service(
                resource("/login")
                    .route(route().guard(Post()).to(login))
                    .route(
                        route()
                            .guard(Not(Post()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    ),
            )
            .service(
                resource("/logout")
                    .route(
                        route()
                            .guard(fn_guard(authorized_guard))
                            .guard(Post())
                            .to(logout),
                    )
                    .route(
                        route()
                            .guard(Not(Post()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    )
                    .default_service(route().to(unauthorized)),
            )
            .service(
                resource("/settings_update_time").route(
                    route()
                        .guard(Post())
                        .guard(fn_guard(authorized_guard))
                        .to(update_time),
                ),
            )
            .service(
                resource("/user_menu")
                    .route(get().guard(fn_guard(authorized_guard)).to(get_user_menu))
                    .route(
                        route()
                            .guard(Not(Get()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    )
                    .default_service(route().to(unauthorized)),
            )
            .service(
                resource("/user_settings")
                    .route(
                        post()
                            .guard(fn_guard(authorized_guard))
                            .to(set_user_settings),
                    )
                    .route(
                        get()
                            .guard(fn_guard(authorized_guard))
                            .to(get_user_settings),
                    )
                    .route(
                        route()
                        .guard(fn_guard(authorized_guard))
                            .guard(Any(Not(Get())).or(Not(Post())))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    )
                    .default_service(route().to(unauthorized)),
            )
            .service(
                resource("/order_dish")
                    .route(post().guard(fn_guard(authorized_guard)).to(order_dish))
                    .route(
                        route()
                            .guard(Not(Post()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    )
                    .default_service(route().to(unauthorized)),
            )
            .service(
                resource("/save_orders")
                    .route(post().guard(fn_guard(authorized_guard)).to(save_orders))
                    .route(
                        route()
                            .guard(Not(Post()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    )
                    .default_service(route().to(unauthorized)),
            )
            .service(resource("/user_status").route(get().to(user_status)))
            .service(resource("/cantine_history").route(get().guard(Not(fn_guard(authorized_guard))).to(cantine_history_query)).route(get().guard(fn_guard(authorized_guard)).to(authorized_cantine_history_query)))
            .service(
                resource("/settings_query")
                    .route(get().guard(fn_guard(authorized_guard)).to(settings_query))
                    .route(
                        route()
                            .guard(Not(Get()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    )
                    .default_service(route().to(unauthorized)),
            )
    })
    .bind((
        env::var("HOST_ADDRESS").unwrap(),
        env::var("PORT").unwrap().parse().unwrap(),
    ))?
    .run()
    .await
}

fn authorized_guard(context: &GuardContext) -> bool {
    match context.get_session().get::<String>("username") {
        Ok(Some(_)) => {
            return true;
        }
        _ => {
            return false;
        }
    }
}
async fn user_status(session: Session) -> impl Responder {
    match session.get::<String>("username") {
        Ok(Some(username)) => {
            return succes("logged as", &format!(r#""{}""# , username));
        }
        _ => {
            return HttpResponse::Unauthorized()
                .body(format!(r#"{{"message":"not authenticated"}}"#));
        }
    }
}
async fn update_time(session: Session, state: Data<Mutex<AppState>>) -> impl Responder {
    let time = state
        .lock()
        .unwrap()
        .db_client
        .get_settings_update_time(session.get::<String>("id").unwrap().unwrap().as_str())
        .await;
    match time {
        Ok(time) => match time {
            Some(time) => succes(
                "settings_update_time",
                serde_json::to_string(&time).unwrap().as_str(),
            ),
            None => {
                return HttpResponse::NoContent().body(format!(
                    r#"{{"message":"no settings found for given user"}}"#,
                ));
            }
        },
        Err(_) => server_error("server error occurred while loading user data"),
    }
}
async fn get_user_menu(state: Data<Mutex<AppState>>, session: Session) -> impl Responder {
    let menu = state
        .lock()
        .unwrap()
        .strava_clients
        .get(&session.get::<String>("session_id").unwrap().unwrap())
        .unwrap()
        .get_menu()
        .await
        .unwrap();
    return succes("menu", serde_json::to_string(&menu).unwrap().as_str());
}
async fn login(req_body: String, session: Session, state: Data<Mutex<AppState>>) -> impl Responder {
    println!("{:?}", req_body);
    match serde_json::from_str::<User<'_>>(&req_body) {
        Ok(user_data) => {
            let client = match StravaClient::new_with_settings(Config {
                settings: HashMap::from([("data_source".to_owned(), "api".to_owned())]),
            })
            .await {
                Ok(client) => client,
                Err(_) => {
                    return server_error("server error occurred during logging in");
                }
            };
            match client.login(&user_data).await {
                Ok(user) => {
                    let id = format!("{}{}", user_data.username, user_data.cantine);
                    let session_id = format!("{}{}", id, rand::thread_rng().gen::<i16>());
                    session.renew();
                    session.insert("id", id.clone()).unwrap();
                    session.insert("session_id", session_id.clone()).unwrap();
                    session.insert("username", user.username.clone()).unwrap();

                    let mut state = state.lock().unwrap();
                    println!("{:?}",id);
                    state.strava_clients.insert(session_id.clone(), client);
                    match state.db_client.get_settings(&id).await {
                        Ok(val) => match val {
                            Some(settings) => {
                                println!("{:?}", settings);
                                let automatic_client = strava_client::automatic_client::AutomaticStravaClient::new_with_existing_request_builder(
                                    settings,                              
                                    state.strava_clients.get(&session_id).unwrap().request_builder.clone(),
                                );
                                let _ = automatic_client.cancel_orders().await.unwrap();   
                            }
                            None => ()
                        }
                        Err(_) =>()
                    }                 
                    
                    return HttpResponse::Ok().body(format!(
                        r#"{{"message":"succesfully logged in","user":{}}}"#,
                        serde_json::to_string(&user).unwrap()
                    ));
                }
                Err(_) => {
                    return HttpResponse::Unauthorized()
                        .body(format!(r#"{{"message":"incorrect user credentials"}}"#));
                }
            }
        }
        Err(_) => {
            return server_error("server error occurred during logging in");
        }
    }
}
async fn get_user_settings(session: Session, state: Data<Mutex<AppState>>) -> impl Responder {
    let settings = state
        .lock()
        .unwrap()
        .db_client
        .get_settings(
            session
                .get::<String>("id")
                .unwrap()
                .unwrap()
                .as_str(),
        )
        .await;
    match settings {
        Ok(settings) => match settings {
            Some(settings) => {
                println!("{:?}", serde_json::to_string(&settings));
                return succes(
                    "settings",
                    serde_json::to_string(&settings).unwrap().as_str(),
                );
            }
            None => {
                return HttpResponse::NoContent().finish();
            }
        },
        Err(_) => server_error("server error occurred while loading user data"),
    }
}
async fn set_user_settings(
    session: Session,
    req_body: String,
    state: Data<Mutex<AppState>>,
    query: Query<SetSettingsUrlString> 
) -> impl Responder {
    let query_string = query.into_inner();
    let item = match query_string.list.as_str() {
        "blacklist" | "whitelist" => { match serde_json::from_str::<DishDBEntry>(&req_body) {
            Ok(val) => Ok(SettingsData::Dish(val)),
            Err(_) => Err(())
        }},
        "strategy" =>  { match serde_json::from_str::<String>(&req_body) {
            Ok(val) => Ok(SettingsData::Strategy(val)),
            Err(_) => Err(())
        }}, 
        "allergens" =>  { match serde_json::from_str::<String>(&req_body) {
            Ok(val) => Ok(SettingsData::Allergen(val)),
            Err(_) => Err(())
        }},
        _ => {
            return HttpResponse::BadRequest()
                .body(format!(r#"{{"message":"Požadovaná položka neexistuje"}}"#));
        }
        
    };
   match item {
        Ok(item) => {
            match state
                .lock()
                .unwrap()
                .db_client
                .update_user_settings(
                    session.get::<String>("id").unwrap().unwrap().as_str(),
                    &query_string.list,
                    &query_string.action,
                    item
                )
                .await
            {
                Ok(_) => {
                    return succes("message", r#""settings was succesfully saved""#);
                }
                Err(e) => {
                    return server_error(&e);
                }
            }
        }
        Err(_) => {
            return HttpResponse::BadRequest()
                .body(format!(r#"{{"message":"invalid request body"}}"#));
        }
    }
    
}
async fn order_dish(
    req_body: String,
    state: Data<Mutex<AppState>>,
    session: Session,
) -> impl Responder {
    match serde_json::from_str::<OrderDishRequestBody>(req_body.as_str()) {
        Ok(dish_info) => {
            match state
                .lock()
                .unwrap()
                .strava_clients
                .get_mut(&session.get::<String>("session_id").unwrap().unwrap())
                .unwrap()
                .order_dish(dish_info.id, dish_info.status)
                .await
            {
                Ok(account) => {
                    return succes("account", &format!(r#""{}""#, account));
                }
                Err(res) => {
                    return server_error(&res.replace("\r\n", " "));
                }
            }
        }
        Err(_) => {
            return HttpResponse::BadRequest()
                .body(format!(r#"{{"message":"invalid request body"}}"#));
        }
    }
}
async fn save_orders(state: Data<Mutex<AppState>>, session: Session) -> impl Responder {
    match state
        .lock()
        .unwrap()
        .strava_clients
        .get_mut(&session.get::<String>("session_id").unwrap().unwrap())
        .unwrap()
        .save_orders()
        .await
    {
        Ok(_) => {
            return succes("message", r#""orders was succesfully saved""#);
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!(
                r#"{{"message":"{}", "account": "{}"}}"#,
                e.error.replace("\r\n", " "),
                e.account
            ));
        }
    }
}
async fn settings_query(
    query: Query<SettingsQueryUrlString>,
    state: Data<Mutex<AppState>>,
    session: Session,
) -> impl Responder {
    let query_string = query.into_inner();
    let settings_query = state
        .lock()
        .unwrap()
        .db_client
        .query_settings(
            session.get::<String>("id").unwrap().unwrap().as_str(),
            &query_string.query, &query_string.list
        )
        .await;
    match settings_query {
        Ok(result) => match serde_json::to_string(&result) {
            Ok(json) => {
                return succes("result", &json);
            }
            Err(_) => {
                return server_error("Došlo k chybě při načítaní dat z databáze.");
            }
        },
        Err(e) => {
            return server_error(&e);
        }
    }
}
/*
async fn get_cantine_history(path: Path<String>, state: Data<Mutex<AppState>>) -> impl Responder {
    let cantine_id = path.into_inner();
    let history = state
        .lock()
        .unwrap()
        .db_client
        .get_cantine(&cantine_id)
        .await;
    match history {
        Ok(history) => match history {
            Some(history) => succes(
                "cantine_history",
                serde_json::to_string(&history).unwrap().as_str(),
            ),
            None => {
                return HttpResponse::NoContent().finish();
            }
        },
        Err(_) => server_error("server error occurred while loading cantine data"),
    }
}
*/
async fn cantine_history_query(
    query: Query<DBHistoryQueryUrlString>,
    state: Data<Mutex<AppState>>,
) -> impl Responder {
    let query_string = query.into_inner();
    let history = state
        .lock()
        .unwrap()
        .db_client
        .query_cantine_history(&query_string.cantine_id, &query_string.query)
        .await;
    match history {
        Ok(data) => {
            return succes("result", serde_json::to_string(&data).unwrap().as_str());
        }
        Err(_) => {
            return server_error("Došlo k chybě při načítaní dat z databáze");
        }
    }
}
async fn authorized_cantine_history_query( 
    query: Query<AuthorizedDBHistoryQueryUrlString>,
    state: Data<Mutex<AppState>>,
    session: Session
) -> impl Responder {
    let query_string = query.into_inner();
    let history = state
        .lock()
        .unwrap()
        .db_client
        .query_cantine_history_for_authorized_user(&query_string.cantine_id, &query_string.query,&query_string.list,&session.get::<String>("id").unwrap().unwrap())
        .await;
    match history {
        Ok(data) => {
            return succes("result", serde_json::to_string(&data).unwrap().as_str());
        }
        Err(e) => {
            println!("{:?}", e);
            return server_error("Došlo k chybě při načítaní dat z databáze");
        }
    }
}
async fn logout(session: Session, state: Data<Mutex<AppState>>) -> impl Responder {
    let username = session.get::<String>("username").unwrap().unwrap();
    state
        .lock()
        .unwrap()
        .strava_clients
        .remove(&session.get::<String>("session_id").unwrap().unwrap());
    session.purge();
    return succes(
        "message",
        &format!(r#""{} was succesfully logged out""#, username),
    );
}
async fn unauthorized() -> impl Responder {
    return HttpResponse::Unauthorized().body(format!(r#"{{"message":"action forbiden"}}"#,));
}
fn server_error(message: &str) -> HttpResponse {
    return HttpResponse::InternalServerError().body(format!(r#"{{"message":"{}"}}"#, message));
}
fn succes(key: &str, value: &str) -> HttpResponse {
    return HttpResponse::Ok().body(format!(r#"{{"{}": {} }}"#, key, value));
}
