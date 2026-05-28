
use serde::Deserialize;
use clap::Parser;
use std::env;

///Q&A web service API
#[derive(Parser, Debug, Default, Deserialize, PartialEq)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    #[clap(short, long, default_value = "info")]
    pub log_level: String,
    #[clap(long, default_value = "localhost")]
    pub db_host: String,
    #[clap(long, default_value = "5432")]
    pub db_port: u16,
    #[clap(long, default_value = "rustwebdev")]
    pub db_name: String,
    #[clap(long, default_value = "postgres")]
    pub db_user: String,
    #[clap(long, default_value = "1qaz!QAZ")]
    pub db_password: String,
    #[clap(short, long, default_value = "8080")]
    pub port: u16,
}
impl Config {
    pub fn new() -> Result<Config, handle_errors::Error> {
        dotenv::dotenv().ok();
        let config = Config::parse();

        if env::var("BAD_WORDS_API_KEY").is_err() {
            panic!("BadWords API key not set");
        }

        if env::var("PASETO_KEY").is_err() {
            panic!("PASETO_KEY not set");
        }

        let port = std::env::var("PORT")
            .ok()
            .map(|val| val.parse::<u16>())
            .unwrap_or(Ok(config.port))
            .map_err(handle_errors::Error::ParseError)?;

        let db_user =
            env::var("POSTGRES_USER").unwrap_or(config.db_user.to_owned());
        let db_password = env::var("POSTGRES_PASSWORD").unwrap_or(config.db_password.to_owned());
        let db_host =
            env::var("POSTGRES_HOST").unwrap_or(config.db_host.to_owned());
        let db_port = env::var("POSTGRES_PORT")
            .unwrap_or(config.db_port.to_string());
        let db_name =
            env::var("POSTGRES_DB").unwrap_or(config.db_name.to_owned());

        Ok(Config {
            log_level: config.log_level,
            port,
            db_user,
            db_password,
            db_host,
            db_port: db_port
                .parse::<u16>()
                .map_err(handle_errors::Error::ParseError)?,
            db_name,
        })
    }
}