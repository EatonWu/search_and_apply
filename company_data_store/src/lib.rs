extern crate company_common;

use std::collections::HashMap;
use company_common::ProcessedCompany;

pub struct CompanyDataStore {
    total_entries: usize,
    websites_unassigned: usize,
    career_pages_unassigned: usize,
    data_map: HashMap<usize, ProcessedCompany> // CID to ProcessedCompany entry
}

impl CompanyDataStore {
    pub fn new() -> CompanyDataStore {
        CompanyDataStore {
            total_entries: 0,
            websites_unassigned: 0,
            career_pages_unassigned: 0,
            data_map: HashMap::new(),
        }
    }

    pub fn add_company(&mut self, company: ProcessedCompany) {

    }

    pub fn add_alias(&mut self, cid: usize, alias: String) {
        let company = self.data_map.get_mut(&cid).unwrap();
        company.company_aliases.push(alias);
    }

    pub fn contains(&mut self, cid: usize) -> bool {
        self.data_map.contains_key(&cid)
    }

    pub fn print_stats(&self) {
        println!("Total entries: {}", self.total_entries);
        println!("Websites unassigned: {}", self.websites_unassigned);
        println!("Career pages unassigned: {}", self.career_pages_unassigned);
    }
}