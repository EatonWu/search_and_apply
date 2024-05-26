// A good start would probably be just to iterate over all the companies

use std::collections::HashMap;
use std::io::{BufRead, Write};
use reqwest::header;
use chrono;
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use company_data_store::CompanyDataStore;
use company_common::{Company, ProcessedCompany};

/// This function gets the date of the company.idx file
/// The relevant part of the header for this file is 2lines long, and contains:
/// Description
/// Last Data Received
pub fn get_idx_file_date() -> Result<chrono::NaiveDate, Box<dyn std::error::Error>> {
    let file = std::fs::File::open("@unprocessed_data/company.idx")?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();
    let mut date = String::new();
    // ignore first line
    lines.next();
    // get the second line
    if let Some(Ok(line)) = lines.next() {
        date = line;
    }
    // string is now of the format "Last Data Received: MONTH DD, YYYY", parse out into datetime
    let date = date.split(": ").collect::<Vec<&str>>();
    let date = date[1];
    // remove whitespace
    let date = date.trim();
    println!("{}", date);
    let date = chrono::NaiveDate::parse_from_str(date, "%B %d, %Y");
    return match date {
        Ok(date) => Ok(date),
        Err(e) => {
            println!("Error: {:?}", e);
            Err(Box::new(e))
        }
    };
}

pub fn get_companies_from_idx() -> Result<Vec<Company>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open("@unprocessed_data/company.idx")?;
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();
    // ignore first 10 lines
    for _ in 0..10 {
        lines.next();
    }
    let mut all_companies = vec![];

    // parse out the company names and the CIK numbers, which we'll use as hash keys.
    for line in lines {
        let line = line?;
        // company names are at most 60 characters long, but the first column seems to be 63 characters long
        let company_name = &line[0..62].trim();
        let form_numbers = &line[62..74].trim();
        let cik = &line[74..86].trim();
        let date = &line[86..98].trim();
        let file_name = &line[98..].trim();

        let company = Company::new(
            company_name.to_string(),
            cik.parse::<usize>().unwrap(),
            form_numbers.to_string(),
            date.to_string(),
            file_name.to_string(),
        );
        // dbg!(company);
        all_companies.push(company);
    }
    Ok(all_companies)
}

fn add_partition_to_index(companies: &[Company], current_index: usize, index_map: &mut HashMap<usize, usize>) {
    for company in companies {
        index_map.insert(company.cik, current_index);
    }
}

/// Partitions the full company list into `partition` partitions and saves them into jsons.
/// This function will also construct a json index from CIKs to their respective json file.
pub fn separate_and_save_companies(companies: Vec<Company>, partitions: usize, dir: &str)
                                                        -> Result<(), Box<dyn std::error::Error>> {
    let partition_size = companies.len() / partitions;
    let mut index = HashMap::new();
    for i in 0..partitions {
        let start = i * partition_size;
        let end = if i == partitions - 1 {
            companies.len()
        } else {
            (i + 1) * partition_size
        };
        let partition = &companies[start..end];

        add_partition_to_index(partition, i, &mut index);

        let filename = format!("{}/companies_{}.json", dir, i);
        save_companies_to_json(partition.to_vec(), &filename)?;
    }
    Ok(())
}

pub fn process_raw_data(companies: Vec<Company>){
    let mut data_store = CompanyDataStore::new();
    for company in companies {
        if data_store.contains(company.cik) {
            data_store.add_alias(company.cik, company.name);
            continue;
        }

        let processed_company = ProcessedCompany::new(
            company.cik,
            vec![company.name],
            None,
            None,
        );

        data_store.add_company(processed_company);
    }
}

pub fn save_companies_to_json(companies: Vec<Company>, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // save companies pretty printed
    let json = serde_json::to_string_pretty(&companies)?;
    let mut file = std::fs::File::create(filename)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

// This function downloads the master list of companies from the SEC
pub async fn get_company_idx_file_from_sec() -> Result<(), Box<dyn std::error::Error>>{

    // check if file already exists and is up to date
    if std::path::Path::new("@unprocessed_data/company.idx").exists() {
        println!("File already exists");
        let date = get_idx_file_date()?;
        let now = Utc::now();
        // compare day/month/year
        if date.year() == now.year() {
            println!("File is up to date");
            return Ok(());
        }
        else {
            println!("File is not up to date!");
        }
    }

    // check if directory exists
    if !std::path::Path::new("@unprocessed_data").exists() {
        std::fs::create_dir("@unprocessed_data")?;
    }

    // TODO: Might need to find a way to automate finding the latest company.idx file
    let link = "https://www.sec.gov/Archives/edgar/full-index/2024/QTR2/company.idx";
    let mut request = header::HeaderMap::new();

    // SEC API header setup for the user agent
    let user_agent = "Eaton Wu eatonwu100@hotmail.com";
    let host = "www.sec.gov";
    let accept = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";

    request.insert(header::USER_AGENT, user_agent.parse().unwrap());
    request.insert(header::HOST, host.parse().unwrap());
    request.insert(header::ACCEPT, accept.parse().unwrap());

    let mut request_builder = reqwest::Client::new().request(reqwest::Method::GET, link);
    let header_request = request_builder.headers(request);
    let response = header_request.send().await?;
    // save response to a file
    let body = response.text().await?;
    let mut file = std::fs::File::create("@unprocessed_data/company.idx")?;
    file.write_all(body.as_bytes())?;
    Ok(())
}
