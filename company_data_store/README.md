# Purpose

### Caching problem strikes again!

We want to find companies that are large or small, startup or listed,
and we want to do it quickly and ideally without duplicating the data.

### Solution

A serializable hashmap would be ideal with this.
serde_json is a crate that gives this functionality.

### Design

- [ ] `CompanyDataStore` is a struct that contains a `HashMap<String, CompanyData>`, 
where String is the company name.
- [ ] `CompanyData` is a struct that contains a `String` for the company name,
a `String` for the company website, and a `String` for the company career page.
The keys for the hashmap are the company names.

- [ ] For storage and retrieval, some kind of paging system would be nice.
This is a feature that could be added later, if size of the
cache/hashmap becomes a problem.