use company_common::Company;
use company_data_store::CompanyDataStore;
use company_scraper::*;

#[test]
fn retrieve_company_test() {
    let apple_cik = 320193;
    let good_data_store = CompanyDataStore::new();
    let mut good_data_store = match good_data_store {
        Ok(store) => {
            println!("Connected to database");
            store
        },
        Err(e) => {
            println!("Error: {:?}", e);
            assert!(false);
            return;
        }
    };
    let processed_company = good_data_store.get_company_by_cik(apple_cik);
    match processed_company {
        Ok(company) => {
            println!("Successfully retrieved company by CIK");
            println!("{:?}", company);
            assert!(true);
        },
        Err(e) => {
            println!("Error: {:?}", e);
            assert!(false);
            return;
        }
    }
}

#[test]
fn duplicate_pk_cik_test() {
   let data_store = CompanyDataStore::new();
    assert!(data_store.is_ok());
    let mut data_store = data_store.unwrap();
    let cik = 320193;
    let company = data_store.get_company_by_cik(cik);
    assert!(company.is_ok());
    let company = company.unwrap();

    // insert the company back into data_store
    let result = data_store.add_company(company, false);
    assert!(result.is_err());
    let error = result.err().unwrap();
    println!("Error: {:?}", error);
}