#[cfg(test)]
mod website_discovery_tests {
    use website_discovery::*;

    #[tokio::test]
    async fn discover_website_test() {
        let website_discoverer = WebsiteDiscoverer::new().await;
        assert!(website_discoverer.is_ok());
        let mut website_discoverer = website_discoverer.unwrap();
        let res = website_discoverer.discover_website().await;
        assert!(res.is_ok());
    }
}