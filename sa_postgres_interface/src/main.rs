use sa_postgres_interface::{create_table, establish_connection};

pub fn main() {
    let mut client = establish_connection();
    match &client {
        Ok(_) => {
            println!("Connected to database");

        },
        Err(e) => {
            println!("Error: {:?}", e);
            return;
        }
    }
    let mut client = client.unwrap();
    let res = create_table(&mut client, "processed_companies", "");
    match res {
        Ok(_) => {
            println!("Table created successfully");
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

}