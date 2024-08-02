use company_data_store::{CompanyDataStore};
use anyhow::{bail, Error, Result};
use serp_service::{GoogleSerpService};



pub struct WebsiteDiscoverer {
    pub company_data_store: CompanyDataStore,
    pub serp_service: GoogleSerpService,
}

impl WebsiteDiscoverer {
    pub async fn new() -> Result<WebsiteDiscoverer, Error> {
        println!("Instantiating WebsiteDiscoverer");
        let company_data_store = CompanyDataStore::new().await?;
        Ok(WebsiteDiscoverer {
            company_data_store,
            serp_service: GoogleSerpService::new(None),
        })
    }

    pub async fn discover_website(&mut self) -> Result<(), Error> {
        // 1. grab a single undiscovered website from the data store
        // (get a sid that doesn't have any entries in the companywebsites table)
        let (sid, company) = self.company_data_store.get_next_undiscovered_company().await?;
        println!("Company: {:?}", company);

        // 2. search for the company name on google
        let query = construct_query(&company.get_company_name()?);
        let search_results = self.serp_service.search_query(&query).await?;

        for (title, url) in search_results {
            println!("Title: {}, URL: {}", title, url);
        }
        Ok(())
    }

    pub async fn discover_and_upload(&mut self) -> Result<(), Error> {
        // begin transaction
        // 1. grab a single undiscovered website from the data store
        // (get a sid that doesn't have any entries in the companywebsites table)
        let (sid, company) = self.company_data_store.get_next_undiscovered_company().await?;
        println!("Company: {:?}", company);

        // 2. search for the company name on google
        let query = construct_query(&company.get_company_name()?);
        let search_results = self.serp_service.search_query(&query).await;
        let search_results = match search_results {
            Ok(v) => v,
            Err(serp_service::SerpServiceError::JsonFailedError) => {
            // but if error is "JSON response is not an object", then we should
            // delete the company from the company table
                let result = self.company_data_store.delete_company(&sid, false).await;
                match result {
                    Ok(_) => {},
                    Err(e) => {
                        bail!("Error: {:?}", e);
                    }
                }
                return Ok(());
            }
            _ => {
                bail!("Error: {:?}", search_results);
            }
        };

        for (title, url) in &search_results {
            println!("Title: {}, URL: {}", title, url);
        }

        // 3. upload the search results to the data store
        for (title, url) in &search_results {
            let result = self.company_data_store.add_website(&sid, title, url, false, false).await;
            match result {
                Ok(_) => {},
                Err(e) => {
                    bail!("Error: {:?}", e);
                }
            }
        }
        Ok(())
    }

    // the intent is to spawn a thread for this function, which will spawn
    // tasks to discover websites from the data store, and then upload the
    // discovered websites to the data store. This will hopefully run forever.
    pub async fn discover_websites(mut self) -> Result<(), Error> {
        // spawn a task to discover websites from the data store, and then
        // spawn a task to upload the results to the data store
        loop {
            let discover_websites_task_result = self.discover_and_upload().await;
            match discover_websites_task_result {
                Ok(_) => {},
                Err(e) => {
                    // wait 30 minutes before retrying
                    println!("Waiting 30 minutes before retrying: {:?}", e);
                    tokio::time::sleep(std::time::Duration::from_secs(1800)).await;
                }
            }
        }
    }

    pub async fn discover_specific_company(&mut self, company_name: &str) -> Result<(), Error> {
        let query = construct_query(company_name);
        let search_results = self.serp_service.search_query(&query).await?;
        for (title, url) in search_results {
            println!("Title: {}, URL: {}", title, url);
        }
        Ok(())
    }
}

fn construct_query(query: &str) -> String {
    // remove "corp, llc, inc", etc, from the query, ignoring case
    let return_string =
        query
        .to_lowercase()
            .replace("corp", "")
            .replace("llc", "")
            .replace("inc", "")
            .replace("ltd", "")
            .replace("group", "");

    // append "careers"
    let return_string = format!("{} careers", return_string);
    return_string
}

pub async fn discover_websites_from_data_store() -> Result<(), Error> {
    bail!("Not implemented");
}
