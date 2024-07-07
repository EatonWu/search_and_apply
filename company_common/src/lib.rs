use std::collections::HashSet;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    pub name: String,
    pub cik: Option<i32>,
    form_numbers: String,
    date: String,
    file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedCompany {
    pub cik: Option<i32>,
    pub company_aliases: HashSet<String>,
    pub websites: Option<Vec<String>>,
    pub career_page: Option<String>,
    pub tags: Option<Vec<String>>,
    pub has_captcha: Option<bool>,
    // ticker: Option<String> // probably not necessary
}

impl Company {
    pub fn new(name: String, cik: Option<i32>, form_numbers: String, date: String, file_name: String) -> Company {
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
    pub fn new(cik: Option<i32>,
               company_aliases: HashSet<String>,
               websites: Option<Vec<String>>,
               career_page: Option<String>,
    tags: Option<Vec<String>>,
    has_captcha: Option<bool>) -> ProcessedCompany {
        ProcessedCompany {
            cik,
            company_aliases,
            websites,
            career_page,
            tags,
            has_captcha,
        }
    }
}