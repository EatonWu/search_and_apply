use company_scraper;
use company_scraper::get_company_idx_file_from_sec;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    // get_company_idx_file_from_sec().await?;
    let companies = company_scraper::get_companies_from_idx()?;
    println!("Number of companies: {}", companies.len());
    // save companies to pages, there are approximately 200000 entries
    // 200000 / 20 = 10000 entries per file
    company_scraper::separate_and_save_companies(companies, 20)?;

    // for company in companies {
    //     println!("{:?}", company);
    // }

    Ok(())
}