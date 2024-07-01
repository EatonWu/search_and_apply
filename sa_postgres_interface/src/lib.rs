use std::collections::HashMap;
use std::error::Error;
use std::env;
use postgres::{Client, NoTls};

enum CompanyTables {
    CompanyTable,
    CikToSid,
    CompanyAliases,
    CompanyTags,
    CompanyWebsites,
    CompanyCareerPage,
}

impl CompanyTables {
    fn as_sql(&self) -> &str {
        match self {
            CompanyTables::CompanyTable => {
                "sid SERIAL PRIMARY KEY"
            },
            CompanyTables::CikToSid => {
                "cik INTEGER PRIMARY KEY, sid INTEGER REFERENCES CompanyTable(sid)"
            },
            CompanyTables::CompanyAliases => {
                "CompanyAlias VARCHAR(255), sid INTEGER, \
                PRIMARY KEY (CompanyAlias, sid), FOREIGN KEY (sid) REFERENCES CompanyTable(sid)"
            },
            CompanyTables::CompanyTags => {
                "sid INTEGER, tag VARCHAR(255), \
                PRIMARY KEY (sid, tag), FOREIGN KEY (sid) REFERENCES CompanyTable(sid)"
            },
            CompanyTables::CompanyWebsites => {
                "sid INTEGER, website_link VARCHAR(255), has_captcha BOOLEAN, \
                PRIMARY KEY (sid, website_link), FOREIGN KEY (sid) REFERENCES CompanyTable(sid)"
            },
            CompanyTables::CompanyCareerPage => {
                "sid INTEGER PRIMARY KEY, career_page_link VARCHAR(255)"
            },
        }
    }

    fn as_str(&self) -> &str {
        match self {
            CompanyTables::CompanyTable => {
                "CompanyTable"
            },
            CompanyTables::CikToSid => {
                "CikToSid"
            },
            CompanyTables::CompanyAliases => {
                "CompanyAliases"
            },
            CompanyTables::CompanyTags => {
                "CompanyTags"
            },
            CompanyTables::CompanyWebsites => {
                "CompanyWebsites"
            },
            CompanyTables::CompanyCareerPage => {
                "CompanyCareerPage"
            },
        }
    }
}

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
/// @param client: the postgres client
/// @param table_name: the name of the table to create
/// @param attributes: the attributes of the table
/// @param dry_run: if true, will print the queries instead of executing them
pub fn create_table(client: &mut Client, table_name: &str, attributes: &str, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let query = format!("CREATE TABLE IF NOT EXISTS {} ({})", table_name, attributes);
    if dry_run {
        println!("{}", query);
        return Ok(());
    }

    client.execute(&query, &[])?;
    Ok(())
}

/// Create all the tables necessary for the company data
/// @param client: the postgres client
/// @param dry_run: if true, will print the queries instead of executing them
pub fn initialize_database(client: &mut Client, dry_run: bool) -> Result<(), Box<dyn Error>> {
    let tables = vec![
        CompanyTables::CompanyTable,
        CompanyTables::CikToSid,
        CompanyTables::CompanyAliases,
        CompanyTables::CompanyTags,
        CompanyTables::CompanyWebsites,
        CompanyTables::CompanyCareerPage,
    ];
    for table in tables {
        let res = create_table(client, table.as_str(), table.as_sql(), dry_run);
        match res {
            Ok(_) => {
                println!("Table {} created", table.as_str());
            },
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
    Ok(())
}