use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::fs;
use strava_client::automatic_client::AutomaticStravaClient;
use strava_client::data_struct::User;
use strava_client::request_builder::RequestBuilder;

macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                log::error!("{}", e);
                continue;
            }
        }
    };
    ($res:expr, $msg:expr) => {
        match $res {
            Ok(val) => val,
            Err(_) => {
                log::error!("{}", $msg);
                continue;
            }
        }
    };
}
macro_rules! skip_none {
    ($res:expr, $msg:expr) => {
        match $res {
            Some(val) => val,
            None => {
                log::error!($msg);
                continue;
            }
        }
    };
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => {
                continue;
            }
        }
    };
}
#[tokio::main]
async fn main() {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%d %H:%M:%S)}] [{l}]  - {m}\n",
        )))
        .build("logs/output.log")
        .expect("Chyba při zápisu logů na disk");

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .expect("Chyba při incializaci loggeru");
    log4rs::init_config(config).expect("Chyba při inicializaci loggeru");

    let config =  match fs::read_to_string("../auto-canceling.toml")
    {
        Ok(config) => match config.parse::<toml::Value>() {
            Ok(config) => config,
            Err(_) => {
                log::error!("{}", "Formát konfiguračního souboru auto-canceling.toml je neplatný");
                return;
            }
        },
        Err(_) => {
            log::error!("{}", "Nelze načíst konfigurační soubor auto-canceling.toml");
            return;
        }
    };
    let users = match config.get("users") {
        Some(users) => match users.as_array() {
            Some(users) => users,
            None => {
                log::error!("{}", "Chyba formátu konfiguračního souboru - chybí seznam uživatelů [[users]]");
                return;
            }
        },
        None => {
            log::error!("{}", "Chyba formátu konfiguračního souboru - chybí seznam uživatelů [[users]]");
            return;
        }
    };
    for user in users {
        let user = skip_none!(user.as_table(), "Chyba formátu konfiguračního souboru - chybný formát seznamu uživatelů ");
        let username = skip_none!(skip_none!(user.get("jmeno"), "Chyba formátu konfiguračního souboru - uživateli chybí uživatelské jméno").as_str());
        let password = skip_fail!(
            keytar::get_password("strava_client", username),
            "Chyba při načítání dat uživatele z keychainu"
        );
        if !password.success {
            log::error!("Chyba při načítání hesla pro uživatele {}", username);
            continue;
        }
        let db_client = RequestBuilder::new();
        let user = User {
            username: username,
            password: &password.password,
            cantine: skip_none!(skip_none!(user.get("cislo"), "Chyba formátu konfiguračního souboru").as_str()),
            lang: "CZ",
            stay_logged: false,
        };
        let _ = skip_fail!(
            db_client.do_db_auth_request(&user).await,
            "Chyba při autentizaci uživatele"
        );
        let _ = skip_fail!(
            db_client.do_login_request(&user).await,
            "Chyba při autentizaci uživatele"
        );
        let settings = skip_fail!(db_client.get_settings().await);
        let _ = skip_fail!(db_client.do_db_logout_request().await);
        skip_fail!(
            AutomaticStravaClient::new_with_existing_request_builder(settings, db_client)
                .cancel_orders()
                .await
        );
        log::info!("Atomatické odhlašování pro uživatele {} úspěšně dokončeno", username);
    }
}
