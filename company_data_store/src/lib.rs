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
    key_list: Vec<usize>, // list of keys to maintain order
    data_map: HashMap<usize, ProcessedCompany> // CIK to ProcessedCompany entry
}

impl CompanyDataStore {
    pub fn new() -> CompanyDataStore {
        // check @data_store directory for currently existing data
        if !std::path::Path::new("@data_store").exists() {
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
                        key_list: vec![],
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
            key_list: vec![],
        }
    }

    fn add_partition_to_index(companies: &[ProcessedCompany], current_index: usize, index_map: &mut HashMap<usize, usize>) {
        for company in companies {
            index_map.insert(company.cik, current_index);
        }
    }

    pub fn get_key_iter(&self) -> impl Iterator<Item=&usize> {
        self.key_list.iter()
    }

    pub fn get_key_iter_mut(&mut self) -> impl Iterator<Item=&mut usize> {
        self.key_list.iter_mut()
    }

    pub fn to_iter(&self) -> impl Iterator<Item = (&usize, &ProcessedCompany)> {
        let key_list_iter = self.get_key_iter();
        key_list_iter.map(move |key| (key, self.data_map.get(key).unwrap()))
    }

    pub fn save_data(&self) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        let mut file = std::fs::File::create("@data_store/data.json").unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    pub fn get_entry(&self, key: &usize) -> Option<&ProcessedCompany>{
        self.data_map.get(&key)
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

    pub fn get_companies(&self) -> Vec<ProcessedCompany> {
        self.data_map.values().cloned().collect()
    }

    pub fn replace_data(&mut self, companies: &Vec<ProcessedCompany>) {
        self.data_map.clear();
        self.key_list.clear();
        for company in companies {
            self.data_map.insert(company.cik, company.clone());
            self.key_list.push(company.cik);
        }
        self.update_stats(companies);
    }

    pub fn update_stats(&mut self, companies: &Vec<ProcessedCompany>) {
        self.total_entries = companies.len();
        self.career_pages_unassigned = {
            let mut count = 0;
            for company in companies {
                if company.career_page.is_none() {
                    count += 1;
                }
            }
            count
        };
        self.websites_unassigned = {
            let mut count = 0;
            for company in companies {
                if company.websites.is_none() {
                    count += 1;
                }
            }
            count
        };
    }

    /// stores the company data into some kind of storage, to later be stored in persistent storage
    pub fn add_company(&mut self, company: ProcessedCompany, company_name: String) {
        // we've got a couple options here; in the interest of simplicity and our use case,
        // we really only ever have at most 200,000 entries, so it's unlikely
        // we'll run into any memory issues, but it might be worth considering
        // doing some kind of paging, which might add some complexity to the code.
        // I'll probably just stick to the serialized hashmap until I feel like I need to change it.

        if self.contains(company.cik) {
            // if the cik already exists, we can add the name as an alias
            self.add_alias(company.cik, company_name);
            return;
        }
        let cik = company.cik.clone();
        self.data_map.insert(cik, company);
        self.key_list.push(cik);
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