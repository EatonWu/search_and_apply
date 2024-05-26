use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    pub name: String,
    pub cik: usize,
    form_numbers: String,
    date: String,
    file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedCompany {
    pub cik: usize,
    pub company_aliases: HashSet<String>,
    pub website: Option<String>,
    pub career_page: Option<String>
    // ticker: Option<String> // probably not necessary
}

impl Company {
    pub fn new(name: String, cik: usize, form_numbers: String, date: String, file_name: String) -> Company {
        Company {
            name,
            cik,
            form_numbers,
            date,
            file_name
        }
    }

}

impl ProcessedCompany {
    pub fn new(cik: usize, company_aliases: HashSet<String>, website: Option<String>, career_page: Option<String>) -> ProcessedCompany {
        ProcessedCompany {
            cik,
            company_aliases,
            website,
            career_page
        }
    }
}