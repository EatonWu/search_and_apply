use company_scraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    // company_scraper::get_companies_from_sec().await?;
    company_scraper::get_companies_from_idx()?;
    Ok(())
}