use std::error::Error;
use hyper_rustls::{HttpsConnectorBuilder};
use serde_json::Value;
use yup_oauth2 as oauth2;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use company_data_store::{CompanyDataStore};

pub struct WebsiteDiscoverer {
    pub company_data_store: CompanyDataStore,
}

impl WebsiteDiscoverer {

}

async fn _search_query(query: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let mut result_vec: Vec<(String, String)> = vec![]; // Tuples containing titles and links
    // deserialize the applicationsecret from serde_json
    let secret_file_path = "website_discovery/assets/google_api_key"; // the JSON obtained from Google Cloud Console
    let secret = oauth2::read_application_secret(secret_file_path).await?;
    let scopes = &["https://www.googleapis.com/auth/cse"];

    // honestly i have no idea what this does
    let connector = HttpsConnectorBuilder::new().with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();

    let client = hyper::Client::builder().build(connector);

    let auth = InstalledFlowAuthenticator::with_client(
        secret,
        InstalledFlowReturnMethod::HTTPRedirect,
        client.clone(),
    ).persist_tokens_to_disk("website_discovery/assets/tokencache.json") // SOME LOCATION TO STORE YOUR TOKEN
        .build()
        .await?;

    let token = auth.token(scopes).await?;
    // println!("Token: {:?}", &token);

    // Obtained via creating a custom search engine
    // https://developers.google.com/custom-search/docs/tutorial/creatingcse
    let cse_id = "4352d533b0a554434";

    let token_string = token.token();
    let final_token;
    match token_string {
        Some(token) => {
            // println!("Token: {}", &token);
            final_token = token;
        },
        None => {
            // println!("Token is None");
            return Err("Token is None".into());
        }
    }

    let query = vec![("q", query), ("cx", cse_id)];

    // Construct a HTTP client with a http authorization header
    let client = reqwest::Client::builder().build()?
        .get("https://www.googleapis.com/customsearch/v1")
        .bearer_auth(final_token)
        .query(&query)
        .send().await?;

    match client.status() {
        reqwest::StatusCode::OK => {},
        _ => {
            return Err("Error: Status code is not OK".into());
        }
    }
    let response = client.text().await?;

    let jsoned_response: Value = serde_json::from_str(&response)?;
    jsoned_response.get("items").unwrap().as_array().unwrap().iter().for_each(|item| {
        let title = item.get("title").unwrap().as_str().unwrap();
        let link = item.get("link").unwrap().as_str().unwrap();
        // println!("Title: {}", title);
        // println!("Link: {}", link);
        result_vec.push((title.to_string(), link.to_string()));
    });
    Ok(result_vec)
}

pub async fn search_query(query: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let maximum_backoff = 64;
    let mut backoff = 1;
    let res = _search_query(query).await;
    match res {
        Ok(v) => {
            Ok(v)
        },

        Err(e) => { // Retry with exponential backoff
            loop {
                if backoff >= maximum_backoff {
                    return Err(e);
                }
                let res = _search_query(query).await;
                match res {
                    Ok(v) => {
                        return Ok(v);
                    },
                    Err(e) => {
                        backoff *= 2;
                        // delay
                        tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;
                    }
                }
            }
        }
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

pub async fn discover_websites_from_data_store() -> Result<(), Box<dyn Error>> {
    let mut data_store = CompanyDataStore::new();
    let mut urls = vec![];
    for key in data_store.get_key_iter() {
        let entry = data_store.get_entry(key);
        // first check if websites vector is not None
        let proc_company;
        match entry {
            Some(entry) => {
                if entry.websites.is_some() {
                    println!("Entry has websites: {:?}", entry.websites);
                    continue;
                }
                println!("Attempting to discover websites for {:?}", entry.company_aliases.iter().next().unwrap());
                proc_company = entry;
            },
            None => {
                println!("Entry is None");
                continue;
            }
        }
        let query = proc_company.company_aliases.iter().next().unwrap();
        // perform query construction
        let query = construct_query(query);

        let res= search_query(&query).await;
        match res {
            Ok(v) => {
                for (title, url) in v {
                    urls.push((title, url));
                }
            },
            Err(e) => { // query failed
                // we should try again every hour
                let backoff = 3600;
                let mut attempts = 0;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;
                    let res = search_query(&query).await;
                    match res {
                        Ok(v) => {
                            for (title, url) in v {
                                urls.push((title, url));
                            }
                            break;
                        },
                        Err(e) => {
                            if attempts >= 25 {
                                break;
                            }
                        }
                    }
                }
            }
        }

        // add half second delay between requests
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    Ok(())
}
