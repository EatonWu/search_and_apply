use hyper_rustls::{HttpsConnectorBuilder};
use serde_json::Value;
use yup_oauth2 as oauth2;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use company_data_store::{CompanyDataStore};
use anyhow::{Error, bail};
use futures::executor::block_on;

pub trait SerpService {
    fn get_serp(&self, query: &str) -> Result<Vec<(String, String)>, Error>;
}

// ------------------------------------------------------------------------------------------------
// Google Stuff
pub struct GoogleSerpService {
    secret_file_path: String,
    cse_id: String,
}

impl SerpService for GoogleSerpService {
    fn get_serp(&self, query: &str) -> Result<Vec<(String, String)>, Error> {
        let mut result_vec: Vec<(String, String)> = vec![]; // Tuples containing titles and links
        // deserialize the applicationsecret from serde_json
        let secret_file_path = &self.secret_file_path; // the JSON obtained from Google Cloud Console
        let secret = block_on(oauth2::read_application_secret(secret_file_path));
        let secret = match secret {
            Ok(secret) => {
                secret
            },
            Err(e) => {
                bail!("Error: {:?}, file_path: {:?}", e, secret_file_path);
            }
        };
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
        ).persist_tokens_to_disk("assets/tokencache.json") // SOME LOCATION TO STORE YOUR TOKEN
            .build();
        let auth = block_on(auth)?;

        let token = block_on(auth.token(scopes))?;
        // println!("Token: {:?}", &token);

        // Obtained via creating a custom search engine
        // https://developers.google.com/custom-search/docs/tutorial/creatingcse

        let token_string = token.token();
        let final_token;
        match token_string {
            Some(token) => {
                // println!("Token: {}", &token);
                final_token = token;
            },
            None => {
                // println!("Token is None");
                bail!("Token is None");
            }
        }

        let query = vec![("q", query), ("cx", &self.cse_id)];

        // Construct a HTTP client with a http authorization header
        let client = reqwest::Client::builder().build()?
            .get("https://www.googleapis.com/customsearch/v1")
            .bearer_auth(final_token)
            .query(&query)
            .send();
        let client = block_on(client)?;

        match client.status() {
            reqwest::StatusCode::OK => {},
            _ => {
                bail!("Error: Status code is not OK");
            }
        }
        let response = client.text();
        let response = block_on(response)?;

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
}

impl GoogleSerpService {
    pub fn new(secret_file: Option<String>) -> Self {
        if secret_file.is_none() {
            GoogleSerpService { // we expect assets directory to be in the highest level
                secret_file_path: secret_file.unwrap_or("assets/google_api_key".to_string()),
                cse_id: "4352d533b0a554434".to_string(),
            }
        }
        else {
            GoogleSerpService {
                secret_file_path: secret_file.unwrap(),
                cse_id: "4352d533b0a554434".to_string(),
            }
        }
    }

    pub async fn search_query(&mut self, query: &str) -> Result<Vec<(String, String)>, Error> {
        let maximum_backoff = 64;
        let mut backoff = 1;
        loop {
            let res = self.get_serp(query);
            match res {
                Ok(v) => {
                    return Ok(v);
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    if backoff > maximum_backoff {
                        bail!("Backoff limit reached");
                    }
                    println!("Retrying in {} seconds", backoff);
                    std::thread::sleep(std::time::Duration::from_secs(backoff));
                    backoff *= 2;
                }
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Bing Stuff

pub struct BingSerpService {
    secret_file_path: String,
    cse_id: String,
}

// ------------------------------------------------------------------------------------------------
// DuckDuckGo Stuff

pub struct DuckDuckGoSerpService {
    secret_file_path: String,
    cse_id: String,
}


