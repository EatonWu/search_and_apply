use company_scraper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    // get_company_idx_file_from_sec().await?;
    let companies = company_scraper::get_companies_from_idx()?;

    let mut data_store = company_scraper::process_raw_data(companies);
    let filtered_companies = company_scraper::filter_data(&data_store,
                                                          vec!["inc", "corp", "llc", "ltd", "group"]);
    data_store.replace_data(&filtered_companies);
    for company in &filtered_companies {
        println!("{:?}", company);
    }
    println!("Total companies: {}", &filtered_companies.len());
    data_store.save_data();
    data_store.print_stats();
    Ok(())
}