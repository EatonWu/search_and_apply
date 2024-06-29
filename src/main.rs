use company_data_store::*;

fn main() {
    let data_store = CompanyDataStore::new();
    let key_list = data_store.get_key_iter();
    for key in key_list {
        let entry = data_store.get_entry(key);
        match entry {
            Some(entry) => {
                println!("{:?}", entry);
            },
            None => {
                println!("No entry found for key: {:?}", key);
            }
        }
    }
}
