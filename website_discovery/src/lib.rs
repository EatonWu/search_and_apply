use hyper_rustls::{HttpsConnectorBuilder};
use serde_json::Value;
use yup_oauth2 as oauth2;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use company_data_store::{CompanyDataStore};
use anyhow::Error;
pub struct WebsiteDiscoverer {
    pub company_data_store: CompanyDataStore,
}

impl WebsiteDiscoverer {
    pub fn new() -> Result<WebsiteDiscoverer, Error> {
        let company_data_store = CompanyDataStore::new()?;
        Ok(WebsiteDiscoverer {
            company_data_store,
        })
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
    return_string
}

pub async fn discover_websites_from_data_store() -> Result<(), Error> {

}
