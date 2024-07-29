use serp_service::{GoogleSerpService, SerpService};

#[tokio::main]
async fn main() {
    // print working directory
    let cwd = std::env::current_dir().unwrap();
    println!("Current working directory: {:?}", cwd);
    let serp_service = GoogleSerpService::new(None);
    match serp_service.get_serp("test") {
        Ok(v) => {
            for (title, url) in v {
                println!("Title: {}, URL: {}", title, url);
            }
        },
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }


}