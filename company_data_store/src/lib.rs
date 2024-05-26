extern crate company_common;
extern crate serde;

use std::collections::HashMap;
use std::io::Write;
use serde::{Deserialize, Serialize};
use company_common::{Company, ProcessedCompany};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyDataStore {
    total_entries: usize,
    websites_unassigned: usize,
    career_pages_unassigned: usize,
    data_map: HashMap<usize, ProcessedCompany> // CID to ProcessedCompany entry
}

impl CompanyDataStore {
    pub fn new() -> CompanyDataStore {
        // check @data_store directory for currently existing data
        if std::path::Path::new("@data_store").exists() {
            // create the directory
            std::fs::create_dir("@data_store").unwrap();
        }
        // check if data.json exists
        if std::path::Path::new("@data_store/data.json").exists() {
            // load the data into the hashmap
            let data = std::fs::read_to_string("@data_store/data.json");
            let data = match &data {
                Ok(data) => { data },
                Err(e) => {
                    println!("Error: {:?}", e);
                    return CompanyDataStore {
                        total_entries: 0,
                        websites_unassigned: 0,
                        career_pages_unassigned: 0,
                        data_map: HashMap::new(),
                    };
                }
            };
            let store = serde_json::from_str::<CompanyDataStore>(&data);
            match store {
                Ok(store) => {
                    return store;
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
            // for each file, load the data into the hashmap
        }
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

    /// stores the company data into some kind of storage, to later be stored in persistent storage
    pub fn add_company(&mut self, company: ProcessedCompany) {
        // we've got a couple options here; in the interest of simplicity and our use case,
        // we really only ever have at most 200,000 entries, so it's unlikely
        // we'll run into any memory issues, but it might be worth considering
        // doing some kind of paging, which might add some complexity to the code.
        // I'll probably just stick to the serialized hashmap until I feel like I need to change it.
        if self.contains(company.cik) {
            // we've already got this company, so we'll just add the alias
            self.add_alias(company.cik, company.company_aliases[0].clone());
            return;
        }
        self.data_map.insert(company.cik, company);
        self.total_entries += 1;
    }

    pub fn add_alias(&mut self, cid: usize, alias: String) {
        let company = self.data_map.get_mut(&cid).unwrap();
        company.company_aliases.insert(alias);
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