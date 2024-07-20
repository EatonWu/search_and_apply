use company_data_store::*;
use postgres::*;

fn main() {
    let dry_run = false;
    let data_store = CompanyDataStore::new();
    let data_store = match data_store {
        Ok(store) => {
            println!("Connected to database");
            store
        },
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    };

    // data_store.create
}
