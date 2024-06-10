use std::error::Error;
use hyper_rustls::{HttpsConnectorBuilder};
use serde_json::Value;
use yup_oauth2 as oauth2;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};


pub async fn search_query(query: &str) -> Result<Vec<(String, String)>, Box<dyn Error>> {
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
    println!("Token: {:?}", &token);

    // Obtained via creating a custom search engine
    // https://developers.google.com/custom-search/docs/tutorial/creatingcse
    let cse_id = "4352d533b0a554434";

    let token_string = token.token();
    let final_token;
    match token_string {
        Some(token) => {
            println!("Token: {}", &token);
            final_token = token;
        },
        None => {
            println!("Token is None");
            return Err("Token is None".into());
        }
    }

    let query = vec![("q", query), ("cx", cse_id)];

    // Construct an HTTP client with a http authorization header
    let client = reqwest::Client::builder().build()?
        .get("https://www.googleapis.com/customsearch/v1")
        .bearer_auth(final_token)
        .query(&query)
        .send().await?;

    let response = client.text().await?;
    let jsoned_response: Value = serde_json::from_str(&response)?;
    jsoned_response.get("items").unwrap().as_array().unwrap().iter().for_each(|item| {
        let title = item.get("title").unwrap().as_str().unwrap();
        let link = item.get("link").unwrap().as_str().unwrap();
        println!("Title: {}", title);
        println!("Link: {}", link);
        result_vec.push((title.to_string(), link.to_string()));
    });
    println!("{:?}", &result_vec);
    Ok(result_vec)
}
