use hyper_rustls::{HttpsConnectorBuilder};
use serde_json::Value;
use yup_oauth2 as oauth2;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use company_data_store::{CompanyDataStore};
use anyhow::{Error, bail, Result};
use futures::executor::block_on;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum SerpServiceError {
    #[error("Status code: {status_code}")]
    StatusError {status_code: u16},
    #[error("Failed to parse JSON")]
    JsonFailedError,
    #[error("Failed to get token")]
    TokenRetrievalError,
    #[error("Authentication error")]
    AuthError,
    #[error("HTTP Client build error")]
    HttpClientBuildError,
    #[error("HTTP Request error")]
    HttpRequestError,
}


// ------------------------------------------------------------------------------------------------
// Google Stuff
pub struct GoogleSerpService {
    secret_file_path: String,
    cse_id: String,
}

impl GoogleSerpService {
    async fn get_serp(&self, query: &str) -> Result<Vec<(String, String)>, SerpServiceError> {
        let mut result_vec: Vec<(String, String)> = vec![]; // Tuples containing titles and links
        // deserialize the applicationsecret from serde_json
        let secret_file_path = &self.secret_file_path; // the JSON obtained from Google Cloud Console
        let secret = oauth2::read_application_secret(secret_file_path).await;
        let secret = match secret {
            Ok(secret) => {
                secret
            },
            Err(e) => {
                return Err(SerpServiceError::TokenRetrievalError);
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
        let auth = auth.await;
        let auth = match auth {
            Ok(auth) => {auth},
            Err(_) => {
                return Err(SerpServiceError::TokenRetrievalError);
            }
        };

        let token = auth.token(scopes).await;
        let token = match token {
            Ok(token) => {
                token
            },
            Err(_) => {
                return Err(SerpServiceError::AuthError);
            }
        };
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
                return Err(SerpServiceError::TokenRetrievalError);
            }
        }

        let query = vec![("q", query), ("cx", &self.cse_id)];

        // Construct a HTTP client with a http authorization header
        let client = reqwest::Client::builder().build();
        let client = match client {
            Ok(client) => {
                client
            },
            Err(_) => {
                return Err(SerpServiceError::HttpClientBuildError);
            }
        };

        let client = client.get("https://www.googleapis.com/customsearch/v1")
            .bearer_auth(final_token)
            .query(&query)
            .send();
        let client = client.await;

        let client = match client {
            Ok(client) => {
                client
            },
            Err(_) => {
                return Err(SerpServiceError::HttpRequestError);
            }
        };

        match client.status() {
            reqwest::StatusCode::OK => {},
            _ => {
                return Err(SerpServiceError::StatusError {status_code: client.status().as_u16()});
            }
        }
        let response = client.text();
        let response = response.await;
        let response = match response {
            Ok(response) => {
                response
            },
            Err(_) => {
                return Err(SerpServiceError::HttpRequestError);
            }
        };

        let jsoned_response: Value = serde_json::from_str(&response).map_err(|_| SerpServiceError::JsonFailedError)?; // hmmm

        let jsoned_response = match jsoned_response.get("items") {
            Some(jsoned_response) => {
                jsoned_response
            },
            None => {
                return Err(SerpServiceError::JsonFailedError);
            }
        };
        let jsoned_response = match jsoned_response.as_array() {
            Some(jsoned_response) => {
                jsoned_response
            },
            None => {
                return Err(SerpServiceError::JsonFailedError);
            }
        };


        jsoned_response.iter().for_each(|item| {
            let title = item.get("title");
            let title = match title {
                Some(title) => {
                    title
                },
                None => {
                    return;
                }
            };

            let title = title.as_str();
            let title = match title {
                Some(title) => {
                    title
                },
                None => {
                    return;
                }
            };

            let link = item.get("link");
            let link = match link {
                Some(link) => {
                    link
                },
                None => {
                    return;
                }
            };
            let link = link.as_str().unwrap();
            println!("Title: {}", title);
            println!("Link: {}", link);
            result_vec.push((title.to_string(), link.to_string()));
        });
        Ok(result_vec)
    }
}

impl GoogleSerpService {
    pub fn new(secret_file: Option<String>) -> Self {
        println!("Instantiating GoogleSerpService");
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

    pub async fn search_query(&self, query: &str) -> Result<Vec<(String, String)>, SerpServiceError> {
        println!("Searching query: {}", query);
        let maximum_backoff = 64;
        let mut backoff = 1;
        loop {
            let res = self.get_serp(query).await;
            match res {
                Ok(v) => {
                    return Ok(v);
                },
                Err(SerpServiceError::JsonFailedError) => {
                    return Err(SerpServiceError::JsonFailedError);
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    if backoff > maximum_backoff {
                        SerpServiceError::StatusError {status_code: 429};
                    }
                    println!("Retrying in {} seconds", backoff);
                    tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
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

}

impl DuckDuckGoSerpService {
    fn get_serp(&self, query: &str) -> Result<Vec<(String, String)>, Error> {
        unimplemented!()
    }
}


