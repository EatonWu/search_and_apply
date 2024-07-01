use sa_postgres_interface::{create_table, establish_connection, initialize_database};

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
    let dry_run = false;
    let res = initialize_database(&mut client, dry_run);
    match res {
        Ok(_) => {
            println!("Database initialized");
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}