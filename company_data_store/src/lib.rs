extern crate company_common;

use std::collections::HashMap;
use std::io::Write;
use company_common::{Company, ProcessedCompany};

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

    fn add_partition_to_index(companies: &[ProcessedCompany], current_index: usize, index_map: &mut HashMap<usize, usize>) {
        for company in companies {
            index_map.insert(company.cik, current_index);
        }
    }

    /// Partitions the full company list into `partition` partitions and saves them into jsons.
    /// This function will also construct a json index from CIKs to their respective json file.
    pub fn separate_and_save_companies(companies: Vec<ProcessedCompany>, partitions: usize, dir: &str)
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

            Self::add_partition_to_index(partition, i, &mut index);

            let filename = format!("{}/companies_{}.json", dir, i);
            Self::save_companies_to_json(partition.to_vec(), &filename)?;
        }
        Ok(())
    }

    pub fn save_companies_to_json(companies: Vec<ProcessedCompany>, filename: &str)
                                                -> Result<(), Box<dyn std::error::Error>> {
        // save companies pretty printed
        let json = serde_json::to_string_pretty(&companies)?;
        let mut file = std::fs::File::create(filename)?;
        file.write_all(json.as_bytes())?;
        Ok(())
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