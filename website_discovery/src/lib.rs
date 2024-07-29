use company_data_store::{CompanyDataStore};
use anyhow::{bail, Error};
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
        let company = self.company_data_store.get_next_undiscovered_company().await?;
        println!("Company: {:?}", company);

        // 2. search for the company name on google
        let query = construct_query(&company.get_company_name()?);
        let search_results = self.serp_service.search_query(&query).await?;

        for (title, url) in search_results {
            println!("Title: {}, URL: {}", title, url);
        }
        Ok(())
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
