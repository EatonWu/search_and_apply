use sa_postgres_interface::establish_connection;

pub fn main() {
    let client = establish_connection();
    match client {
        Ok(client) => {
            println!("Connected to database");
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}