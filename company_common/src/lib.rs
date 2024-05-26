use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    name: String,
    cik: usize,
    form_numbers: String,
    date: String,
    file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedCompany {
    cik: usize,
    company_aliases: Vec<String>,
    website: Option<String>,
    career_page: Option<String>
    // ticker: Option<String> // probably not necessary
}