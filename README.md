# search_and_apply

## What is this?
Repository for my little job search (and apply?) program/bot

## Plans/Requirements
- [ ] How do I locate company names?
- [ ] How do I locate company websites?
- [ ] How do I locate company career pages?
- [ ] How do I distinguish different job postings/navigate to them?
- [ ] How do I apply to a job posting?
- [ ] How do I fill out a job application?
- [ ] How do I submit a job application?

Oh boy.

## Crates

- `search_and_apply`: The main crate
- `company_data_store`: A crate for storing company data, so no duplicate searches are made
- `company_scraper`: A crate for scraping websites for company data
- `job_applier`: A crate for applying to jobs; still debating feasibility,
and whether I should focus on compiling a list of job postings first.
Simplify kind of deals with this for us, if we wanted some human intervention.
