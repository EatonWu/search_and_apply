use company_data_store::*;
use website_discovery::{discover_websites_from_data_store, WebsiteDiscoverer};
use anyhow::{bail, Error};
#[tokio::main]
async fn main() -> Result<(), Error>{
    std::env::set_var("RUST_LIB_BACKTRACE", "0");
    std::env::set_var("RUST_BACKTRACE", "1");
    println!("Working directory: {:?}", std::env::current_dir()?);
    let mut discoverer: Result<WebsiteDiscoverer, Error> = WebsiteDiscoverer::new().await;
    let mut discoverer = match discoverer {
        Ok(discoverer) => discoverer,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return Ok(());
        }
    };
    println!("Instantiated discoverer");
    let res = tokio::spawn(discoverer.discover_websites());
    let res = res.await;
    bail!("Error: {:?}", res);
}
