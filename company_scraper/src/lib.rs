// A good start would probably be just to iterate over all the companies

use std::io::{BufRead, Write};
use reqwest::header;
use chrono;
use chrono::format::{Item, Parsed};
use chrono::{Datelike, FixedOffset, Utc};

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

// This function downloads the master list of companies from the SEC
pub async fn get_companies_from_sec() -> Result<(), Box<dyn std::error::Error>>{

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
