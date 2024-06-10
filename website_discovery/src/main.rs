use company_data_store::*;
use website_discovery::{search_query};

#[tokio::main]
async fn main() {
    let query = "apple";
    let urls = search_query(query).await;
}
