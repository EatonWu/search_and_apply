use std::error::Error;
use std::env;
use postgres::{Client, NoTls};

pub fn establish_connection() ->  Result<Client, Box<dyn Error>>{
    dotenvy::dotenv()?;
    let database_url = env::var("DATABASE_URL")?;
    let client = Client::connect(
        database_url.as_str(),
        NoTls,
    )?;
    Ok(client)
}

// pub fn get_