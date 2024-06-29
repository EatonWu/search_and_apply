use company_data_store::*;
use website_discovery::{discover_websites_from_data_store, search_query};

#[tokio::main]
async fn main() {
    let query = "Northfield Bancorp";
    let res = discover_websites_from_data_store().await;
}
