// A good start would probably be just to iterate over all the companies

use std::io::{BufRead, Write};
use reqwest::header;
use chrono;
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    name: String,
    cik: String,
    form_numbers: String,
    date: String,
    file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedCompany {
    cik: String,
    company_name: String,
    website: Option<String>,
    // ticker: Option<String> // probably not necessary
}


/// This function gets the date of the company.idx file
/// The relevant part of the header for this file is 2lines long, and contains:
/// Description
/// Last Data Received
pub fn get_idx_file_date() -> Result<chrono::NaiveDate, Box<dyn std::error::Error>> {
    let file = std::fs::File::open("@data/company.idx")?;
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
    let test = "May 25, 2024";
    println!("{}", date);
    let date = chrono::NaiveDate::parse_from_str(test, "%B %d, %Y");
    return match date {
        Ok(date) => Ok(date),
        Err(e) => {
            println!("Error: {:?}", e);
            Err(Box::new(e))
        }
    };
}

pub fn get_companies_from_idx() -> Result<Vec<Company>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open("@data/company.idx")?;
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

        let company = Company {
            name: company_name.to_string(),
            cik: cik.to_string(),
            form_numbers: form_numbers.to_string(),
            date: date.to_string(),
            file_name: file_name.to_string(),
        };
        // dbg!(company);
        all_companies.push(company);
    }
    Ok(all_companies)
}

pub fn separate_and_save_companies(companies: Vec<Company>, partitions: usize)
                                                        -> Result<(), Box<dyn std::error::Error>> {
    let partition_size = companies.len() / partitions;
    for i in 0..partitions {
        let start = i * partition_size;
        let end = if i == partitions - 1 {
            companies.len()
        } else {
            (i + 1) * partition_size
        };
        let partition = &companies[start..end];
        let filename = format!("@data/companies_{}.json", i);
        save_companies_to_json(partition.to_vec(), &filename)?;
    }
    Ok(())
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
    if std::path::Path::new("@data/company.idx").exists() {
        println!("File already exists");
        let date = get_idx_file_date()?;
        let now = Utc::now();
        // compare day/month/year
        if date.day() == now.day() && date.month() == now.month() && date.year() == now.year() {
            println!("File is up to date");
            return Ok(());
        }
        else {
            println!("File is not up to date!");
        }
    }

    // check if directory exists
    if !std::path::Path::new("@data").exists() {
        std::fs::create_dir("@data")?;
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
    let mut file = std::fs::File::create("@data/company.idx")?;
    file.write_all(body.as_bytes())?;
    Ok(())
}
