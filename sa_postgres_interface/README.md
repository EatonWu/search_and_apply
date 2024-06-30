# Here's the SQL schema that I've come up with:

The root table is the CompanyTable, which can effectively reference all other tables.
The serial id basically acts as a primary key (hash) for the company.

The reason the CIK isn't the primary key is that there will potentially be sources other than
the SEC that will be used to gather data on companies. The CIK is a unique identifier, but it's
not the only one.

CompanyTable
---
* sid: serial primary key

CikToSid
---
* cik: integer primary key
* sid: integer foreign key references CompanyTable(sid)

CompanyAliases
---
* CompanyAlias: varchar(255) primary key
* sid: integer foreign key references CompanyTable(sid)

CompanyTags
---
* sid: integer primary key
* tag: varchar(255) primary key

CompanyWebsites
---
* sid: integer primary key
* website_link: varchar(255) primary key
* has_captcha: boolean

CompanyCareerPage
---
* sid: integer primary key
* career_page_link: varchar(255)
