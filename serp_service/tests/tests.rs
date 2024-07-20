#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use serp_service::*;
    use super::*;

    #[tokio::test]
    async fn test_get_search_results() {
        // print working directory
        let cwd = std::env::current_dir().unwrap();
        println!("Current working directory: {:?}", cwd);
        // set working directory to parent
        std::env::set_current_dir("..").unwrap();
        let cwd = std::env::current_dir().unwrap();
        println!("Current working directory: {:?}", cwd);

        let query = "test";
        let serp_service = GoogleSerpService::new(None);
        let res = serp_service.get_serp(query);
        // match serp_service.get_serp(query) {
        //     Ok(v) => {
        //         for (title, url) in v {
        //             println!("Title: {}, URL: {}", title, url);
        //         }
        //     },
        //     Err(e) => {
        //         println!("Error: {:?}", e);
        //         assert!(false)
        //     }
        // }
    }
}