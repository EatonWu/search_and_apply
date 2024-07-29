extern crate company_common;
extern crate serde;
extern crate tokio_postgres;
extern crate anyhow;

use std::env;
use company_common::{ProcessedCompany};
use tokio_postgres::*;
use anyhow::{bail, Error};
use std::collections::HashSet;
use std::str::FromStr;

pub enum CompanyTables {
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
                "cik INTEGER PRIMARY KEY, sid INTEGER REFERENCES CompanyTable(sid) ON DELETE CASCADE"
            },
            CompanyTables::CompanyAliases => {
                "CompanyAlias VARCHAR(255), sid INTEGER, \
                PRIMARY KEY (CompanyAlias, sid), FOREIGN KEY (sid) REFERENCES CompanyTable(sid) ON DELETE CASCADE"
            },
            CompanyTables::CompanyTags => {
                "sid INTEGER, tag VARCHAR(255), \
                PRIMARY KEY (sid, tag), FOREIGN KEY (sid) REFERENCES CompanyTable(sid) ON DELETE CASCADE"
            },
            CompanyTables::CompanyWebsites => {
                "sid INTEGER, website_link VARCHAR(255), has_captcha BOOLEAN, \
                PRIMARY KEY (sid, website_link), FOREIGN KEY (sid) REFERENCES CompanyTable(sid) ON DELETE CASCADE"
            },
            CompanyTables::CompanyCareerPage => {
                "sid INTEGER PRIMARY KEY, career_page_link VARCHAR(255),\
                FOREIGN KEY (sid) REFERENCES CompanyTable(sid) ON DELETE CASCADE"
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

pub async fn establish_connection() ->  Result<Client, Error>{
    dotenvy::dotenv()?;
    let database_url = env::var("DATABASE_URL")?;
    println!("Attempting to connect to {}", database_url);
    let (client, connection) = connect(
        &database_url,
        NoTls,
    ).await?;
    println!("Connection established, spawning thread");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    Ok(client)
}

pub struct CompanyDataStore {
    postgres_client: Client,
}

impl CompanyDataStore {
    pub async fn new() -> Result<CompanyDataStore, Error> {
        println!("Establishing connection");
        let client = establish_connection().await?;
        let mut data_store = CompanyDataStore {
            postgres_client: client,
        };
        println!("Connection established\nInitializing database");
        data_store.initialize_database(false).await?;
        println!("Database initialized\nCleaning sids");
        data_store.clean_sids().await?;
        println!("Sids cleaned");
        Ok(data_store)
    }

    pub async fn get_companies(&mut self) -> Result<Vec<ProcessedCompany>, Error> {
        let query = "SELECT sid FROM CompanyTable".to_string();
        let results = self.postgres_client.query(&query, &[]).await?;
        let mut companies = Vec::new();
        for row in results {
            let sid: i32 = row.get(0);
            companies.push(self.construct_processed_company_from_sid(&sid).await?);
        }
        Ok(companies)
    }



    /// Create all the tables necessary for the company data
    /// @param client: the postgres client
    /// @param dry_run: if true, will print the queries instead of executing them
    pub async fn initialize_database(&mut self, dry_run: bool) -> Result<(), Error> {
        let tables = vec![
            CompanyTables::CompanyTable,
            CompanyTables::CikToSid,
            CompanyTables::CompanyAliases,
            CompanyTables::CompanyTags,
            CompanyTables::CompanyWebsites,
            CompanyTables::CompanyCareerPage,
        ];
        for table in tables {
            let res = self.create_table(table.as_str(), table.as_sql(), dry_run).await;
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

    pub async fn insert_into_table(&mut self, table: CompanyTables, values: Vec<&(dyn types::ToSql + Sync)>, dry_run: bool) -> Result<(), Error> {
        let query = format!("INSERT INTO {} VALUES ({})", table.as_str(), values.iter().enumerate().map(|(i, _)| format!("${}", i + 1)).collect::<Vec<String>>().join(", "));
        // println!("{}", query);
        self.postgres_client.execute(&query, &values.to_vec()).await?;
        Ok(())
    }

    /// Perform a delete operation on the company with the given sid.
    pub async fn delete_company(&mut self, sid: &i32, dry_run: bool) -> Result<(), Error> {
        let query = "DELETE FROM CompanyTable WHERE sid = $1".to_string();
        if dry_run {
            println!("{}", query);
            return Ok(());
        }
        self.postgres_client.execute(&query, &[&sid]).await?;
        Ok(())
    }


    /// Creates an entry into the CompanyTable, which is a serial value.
    /// Returns the sid of the newly created company
    pub async fn initialize_company(&mut self, dry_run: bool) -> Result<i32, Error> {
        let query = "INSERT INTO CompanyTable DEFAULT VALUES".to_string();
        if dry_run {
            println!("{}", query);
            return Ok(0);
        }
        self.postgres_client.execute(&query, &[]).await?;

        // get the highest serial
        let query = "SELECT MAX(sid) FROM CompanyTable".to_string();
        let results = self.postgres_client.query(&query, &[]).await?;
        let sid: i32 = results[0].get(0);
        Ok(sid)
    }

    /// Create a table in the database with the given name and attributes,
    /// Should never be called by users
    /// Vulnerable to SQL injection?
    /// @param client: the postgres client
    /// @param table_name: the name of the table to create
    /// @param attributes: the attributes of the table
    /// @param dry_run: if true, will print the queries instead of executing them
    pub async fn create_table(&mut self, table_name: &str, attributes: &str, dry_run: bool) -> Result<(), Error> {
        let query = format!("CREATE TABLE IF NOT EXISTS {} ({})", table_name, attributes);
        if dry_run {
            println!("{}", query);
            return Ok(());
        }

        self.postgres_client.execute(&query, &[]).await?;
        Ok(())
    }

    /// Given some ProcessedCompany, we want to insert the information into the database, with
    /// its various tables (CompanyTable, CikToSid, CompanyAliases, CompanyTags, CompanyWebsites,
    /// CompanyCareerPage.
    ///
    /// Many of these are nullable, seeing as we haven't established the company's tags and
    /// websites yet. (Actually, we're not even going to add rows to their respective tables.
    /// A query on a company's websites will return 0 rows if the company has no websites, ideally.
    pub async fn add_company(&mut self, company: ProcessedCompany, dry_run: bool) -> Result<(), Error> {
        // check if the company already exists
        // TODO 1: Figure out some way to do this if there are different identifiers (CIK, ticker, etc.)
        // TODO 2: Make this a transaction
        let sid = match company.cik {
            Some(cik) => {
                self.cik_exists(&cik).await?
            }
            None => {
                None
            }
        };

        let sid = match sid {
            Some(sid) => {
                if company.cik.is_some() {
                    println!("Company with CIK {} already exists.", company.cik.unwrap());
                }
                sid
            },
            None => {
                // if we can't initialize the company, then add_company fails.
                self.initialize_company(dry_run).await?
            }
        };

        match company.cik {
            Some(cik) => {
               self.insert_into_table(CompanyTables::CikToSid, vec![&cik, &sid], dry_run).await?;
            },
            None => {
                println!("No CIK found for company");
                return Ok(());
            }
        }
        // println!("CIK added for company with sid {}", sid);

        for alias in &company.company_aliases {
            self.add_alias(&sid, alias, dry_run).await?;
        }

        // println!("Alias added for company with sid {}", sid);

        if company.tags.is_some() {
            for tag in company.tags.unwrap() {
                self.add_tag(&sid, tag, dry_run).await?;
            }
        }
        if company.websites.is_some() {
            for website in company.websites.unwrap() {
                self.add_website(&sid, website, false, dry_run).await?;
            }
        }

        if company.career_page.is_some() {
            self.add_career_page(&sid, &company.career_page.clone().unwrap(), dry_run).await?;
        }

        if company.has_captcha.is_some() {
            self.update_captcha_status(&sid, company.career_page.unwrap(), company.has_captcha.unwrap()).await?;
        }
        let first_company_name = match company.company_aliases.iter().next() {
            Some(name) => name,
            None => {
                println!("No company name found");
                return Ok(());
            }
        };
        println!("Company with sid {} and name {} added", sid, first_company_name);
        Ok(())
    }

    pub async fn add_cik(&mut self, cik: i32, sid: i32, dry_run: bool) -> Result<(), Error> {
        self.insert_into_table(CompanyTables::CikToSid, vec![&cik, &sid], dry_run).await
    }

    pub async fn add_tag(&mut self, sid: &i32, tag: String, dry_run: bool) -> Result<(), Error> {
        self.insert_into_table(CompanyTables::CompanyTags, vec![sid, &tag], dry_run).await
    }

    pub async fn add_alias(&mut self, sid: &i32, alias: &String, dry_run: bool) -> Result<(), Error>{
        self.insert_into_table(CompanyTables::CompanyAliases, vec![&alias, sid], dry_run).await
    }

    pub async fn add_website(&mut self, sid: &i32, website: String, has_captcha: bool, dry_run: bool) -> Result<(), Error> {
        self.insert_into_table(CompanyTables::CompanyWebsites, vec![sid, &website, &has_captcha], dry_run).await
    }

    pub async fn update_captcha_status(&mut self, sid: &i32, website: String, has_captcha: bool) -> Result<(), Error> {
        let query = "UPDATE CompanyWebsites SET has_captcha = $1 WHERE sid = $2 AND website_link = $3".to_string();
        self.postgres_client.execute(&query, &[&has_captcha, &sid, &website]).await?;
        Ok(())
    }

    pub async fn add_career_page(&mut self, sid: &i32, career_page: &String, dry_run: bool) -> Result<(), Error> {
        self.insert_into_table(CompanyTables::CompanyCareerPage, vec![sid, &career_page], dry_run).await
    }

    /// Deletes all companies with aliases that DON'T contain any of the strings in the filter
    pub async fn filter_companies_alias(&mut self, filter: Vec<&str>) -> Result<(), Error> {
        let query = "SELECT * FROM CompanyAliases WHERE {}".to_string();
        // create regex pattern that matches on any of the strings using ors
        let filters = filter.iter().map(|x| format!("CompanyAlias NOT ILIKE '%{}%'", x)).collect::<Vec<String>>().join(" AND ");
        let query = query.replace("{}", &filters);
        println!("{}", query);
        let sids = self.postgres_client.query(&query, &[]).await?;
        for row in sids {
            let company_name: String = row.get(0);
            let sid: i32 = row.get(1);
            println!("Deleting company {} with sid {}", company_name, sid);
            // self.delete_company(&sid, false)?;
        }
        Ok(())
    }

    pub async fn construct_processed_company_from_sid(&mut self, sid: &i32) -> Result<ProcessedCompany, Error> {
        let cik = self.get_cik_from_sid(sid).await?;
        let aliases = self.get_aliases_from_sid(sid).await?;
        let tags = self.get_tags_from_sid(sid).await?;
        let websites = self.get_websites_from_sid(sid).await?;
        let career_page = self.get_career_page_from_sid(sid).await?;
        let has_captcha = self.get_captcha_status_from_sid(sid).await?;
        Ok(ProcessedCompany::new(cik, aliases, websites, career_page, tags, has_captcha))
    }

    pub async fn get_cik_from_sid(&mut self, sid: &i32) -> Result<Option<i32>, Error> {
        let query = "SELECT cik FROM CikToSid WHERE sid = $1".to_string();
        let results = self.postgres_client.query(&query, &[&sid]).await?;
        if results.len() == 0 {
            return Ok(None);
        }
        Ok(Some(results[0].get(0)))
    }

    pub async fn get_aliases_from_sid(&mut self, sid: &i32) -> Result<HashSet<String>, Error> {
        let query = "SELECT CompanyAlias FROM CompanyAliases WHERE sid = $1".to_string();
        let results = self.postgres_client.query(&query, &[&sid]).await?;
        let mut aliases = HashSet::new();
        for row in results {
            aliases.insert(row.get(0));
        }
        Ok(aliases)
    }

    pub async fn get_tags_from_sid(&mut self, sid: &i32) -> Result<Option<Vec<String>>, Error> {
        let query = "SELECT tag FROM CompanyTags WHERE sid = $1".to_string();
        let results = self.postgres_client.query(&query, &[&sid]).await?;
        let mut tags = Vec::new();
        for row in results {
            tags.push(row.get(0));
        }
        if tags.len() == 0 {
            return Ok(None);
        }
        Ok(Some(tags))
    }

    /// Checks if a company with the given CIK exists
    /// Returns the sid of the company if it exists
    pub async fn cik_exists(&mut self, cik: &i32) -> Result<Option<i32>, Error> {
        let query = "SELECT * FROM CikToSid WHERE cik = $1 LIMIT 1".to_string();
        let results = self.postgres_client.query(&query, &[&cik]).await?;
        let sid = match results.len() {
            0 => None,
            _ => Some(results[0].get(1)),
        };
        Ok(sid)
    }

    pub async fn get_websites_from_sid(&mut self, sid: &i32) -> Result<Option<Vec<String>>, Error> {
        let query = "SELECT website_link FROM CompanyWebsites WHERE sid = $1".to_string();
        let results = self.postgres_client.query(&query, &[&sid]).await?;
        let mut websites = Vec::new();
        for row in results {
            websites.push(row.get(0));
        }
        if websites.len() == 0 {
            return Ok(None);
        }
        Ok(Some(websites))
    }

    pub async fn get_career_page_from_sid(&mut self, sid: &i32) -> Result<Option<String>, Error> {
        let query = "SELECT career_page_link FROM CompanyCareerPage WHERE sid = $1".to_string();
        let results = self.postgres_client.query(&query, &[&sid]).await?;
        if results.len() == 0 {
            return Ok(None);
        }
        Ok(Some(results[0].get(0)))
    }

    pub async fn get_captcha_status_from_sid(&mut self, sid: &i32) -> Result<Option<bool>, Error> {
        let query = "SELECT has_captcha FROM CompanyWebsites WHERE sid = $1".to_string();
        let results = self.postgres_client.query(&query, &[&sid]).await?;
        if results.len() == 0 {
            return Ok(None);
        }
        Ok(Some(results[0].get(0)))
    }

    pub async fn get_sid_from_cik(&mut self, cik: &i32) -> Result<Option<i32>, Error> {
        let query = "SELECT sid FROM CikToSid WHERE cik = $1".to_string();
        let results = self.postgres_client.query(&query, &[&cik]).await?;
        if results.len() == 0 {
            return Ok(None);
        }
        Ok(Some(results[0].get(0)))
    }

    pub async fn get_sid_from_alias(&mut self, alias: &str) -> Result<Option<i32>, Error> {
        let query = "SELECT sid FROM CompanyAliases WHERE CompanyAlias = $1".to_string();
        let results = self.postgres_client.query(&query, &[&alias]).await?;
        if results.len() == 0 {
            return Ok(None);
        }
        Ok(Some(results[0].get(0)))
    }


    /// Checks if some company already exists (might not need)
    pub async fn contains_cik(&mut self, cik: i32) -> Result<bool, Error>{
        // get single row
        let query = format!("SELECT * FROM {} WHERE cik = $1 LIMIT 1", CompanyTables::CikToSid.as_str());
        let res = self.postgres_client.query(&query, &[&cik]).await?;
        Ok(res.len() > 0)
    }

    pub async fn get_company_by_cik(&mut self, cik: i32) -> Result<ProcessedCompany, Error> {
        let sid = self.get_sid_from_cik(&cik).await?;
        match sid {
            Some(sid) => {
                self.construct_processed_company_from_sid(&sid).await
            },
            None => {
                println!("Company with CIK {} not found", cik);
                bail!("Could not find company with CIK")
            }
        }
    }

    pub async fn get_next_undiscovered_company(&mut self) -> Result<ProcessedCompany, Error> {
        let query = "SELECT sid FROM CompanyTable WHERE sid NOT IN (SELECT sid FROM CompanyWebsites) LIMIT 1".to_string();
        let results = self.postgres_client.query(&query, &[]).await?;
        if results.len() == 0 {
            bail!("No undiscovered companies found");
        }
        let sid: i32 = results[0].get(0);
        println!("Found company with sid {}", sid);
        let result = self.construct_processed_company_from_sid(&sid).await;
        match result {
            Ok(v) => {
                Ok(v)
            },
            Err(e) => {
                println!("Error: {:?}", e);
                bail!("Error constructing processed company from sid");
            }
        }
    }

    /// This deletes all sids from companytable that do not have any entries
    /// in either ciktosid or companyaliases
    pub async fn clean_sids(&mut self) -> Result<(), Error> {
        let query = "DELETE FROM CompanyTable WHERE sid NOT IN (SELECT sid FROM CompanyAliases)".to_string();
        let res = self.postgres_client.execute(&query, &[]).await?;
        println!("Deleted {} rows from CompanyTable", res);
        Ok(())
    }

    pub fn print_stats(&self) {

    }
}