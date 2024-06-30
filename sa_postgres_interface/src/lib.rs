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

/// Create a table in the database with the given name and attributes,
/// Should never be called by users
/// Vulnerable to SQL injection?
pub fn create_table(client: &mut Client, table_name: &str, attributes: &str) -> Result<(), Box<dyn Error>> {
    let query = format!("CREATE TABLE {} ({})", table_name, attributes);
    client.execute(&query, &[])?;
    Ok(())
}

// pub fn get_