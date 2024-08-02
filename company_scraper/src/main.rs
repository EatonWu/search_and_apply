use company_scraper;
use company_scraper::get_company_idx_file_from_sec;
use futures::executor::block_on;
use company_data_store::CompanyDataStore;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LIB_BACKTRACE", "0");
    std::env::set_var("RUST_BACKTRACE", "1");

    // use executor to block on this get_company_idx_file_from_sec function
    let thing = block_on(get_company_idx_file_from_sec());
    match thing {
        Ok(_) => {
            println!("Successfully extracted data from sec file");
        },
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    }

    let companies = company_scraper::get_companies_from_idx();
    let mut good_data_store;
    let dry_run = false;
    match companies {
        Ok(companies) => {
            println!("Successfully extracted companies from idx file");
            let data_store = company_scraper::process_raw_sec_data(companies, dry_run).await;
            match data_store {
                Ok(store) => {
                    good_data_store = store;
                    println!("Successfully processed raw data");
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    return;
                }
            };
        },
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    }
    // good_data_store = CompanyDataStore::new();
    // let mut good_data_store = match good_data_store {
    //     Ok(store) => {
    //         println!("Connected to database");
    //         store
    //     },
    //     Err(e) => {
    //         println!("Error: {:?}", e);
    //         return;
    //     }
    // };

    // remove all companies that don't have an alias
    let filter = vec! [
        "inc",
        "corp",
        "llc",
        "ltd",
        "group",
        "lp",
        "l.p",
        "l.l.c",
        "co",
        "l p",
        "company",
        "bank"
    ];
    let res = good_data_store.filter_companies_alias(filter).await;
    match res {
        Ok(_) => {
            println!("Successfully filtered data");
        },
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    }

    // let apple_cik = 320193;
    // let processed_company = good_data_store.get_company_by_cik(apple_cik);
    // match processed_company {
    //     Ok(company) => {
    //         println!("Successfully retrieved company by CIK");
    //         println!("{:?}", company);
    //     },
    //     Err(e) => {
    //         println!("Error: {:?}", e);
    //         return;
    //     }
    // }
}