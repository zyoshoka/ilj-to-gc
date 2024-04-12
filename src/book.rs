use std::collections::HashMap;

use chrono::{Duration, NaiveDate};
use data_encoding::BASE32HEX;
use scraper::{Html, Selector, ElementRef};

use crate::{calendar::Date, Config};

#[derive(Debug)]
pub struct Book {
    pub is_reserved: bool,
    pub lender: String,
    pub holder: String,
    pub due_date: NaiveDate,
    pub lent_date: NaiveDate,
    pub name: String,
}

pub const RESERVED: &str = "[予約有] ";

impl Book {
    pub fn get_summary(&self) -> String {
        (if self.is_reserved { RESERVED.to_string() } else { "".to_string() }) + &self.name
    }

    pub fn get_id(&self) -> String {
        BASE32HEX
            .encode((self.lent_date.to_string() + &self.name.chars().take(50).collect::<String>()).as_bytes())
            .trim_end_matches('=')
            .to_lowercase()
    }

    pub fn get_start_date(&self) -> Date {
        Date { date: self.due_date }
    }

    pub fn get_end_date(&self) -> Date {
        Date { date: self.due_date + Duration::days(1) }
    }
}

async fn log_in(client: &reqwest::Client, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut map = HashMap::new();
    map.insert("userid", &config.userid);
    map.insert("password", &config.password);

    client.post(config.base_url.clone() + "/comidf.do")
        .form(&map).send().await?
        .text().await?;

    Ok(())
}

pub async fn get_borrowed_books(client: &reqwest::Client, config: &Config) -> Result<Vec<Book>, Box<dyn std::error::Error>> {
    log_in(client, config).await?;

    let mut map = HashMap::new();
    map.insert("listcnt", "20");

    let resp = client.post(config.base_url.clone() + "/lenlst.do")
        .form(&map).send().await?
        .text().await?;

    let document = Html::parse_document(&resp);
    let table_selector = Selector::parse(r#"table[class="opac_data_list_ex"]"#)?;
    let table_rows_selector = Selector::parse("tr")?;
    let table_rows = document.select(&table_selector).next().unwrap()
        .select(&table_rows_selector)
        .map(|el| el.children().filter_map(ElementRef::wrap))
        .skip(1);

    let mut books: Vec<Book> = vec![];
    table_rows.for_each(|row| {
        let rows: Vec<_> = row.map(|el| {
            let inner = el.inner_html();
            let parsed_inner = Html::parse_fragment(&inner);
            let parsed_inner_text = parsed_inner.root_element().text().collect::<Vec<_>>().concat().replace(['\n', '\t'], "");
            parsed_inner_text
        }).collect();

        let book = Book {
            is_reserved: rows[2] == "予約有",
            lender: rows[3].clone(),
            holder: rows[4].clone(),
            due_date: NaiveDate::parse_from_str(&rows[5], "%Y/%m/%d").unwrap(),
            lent_date: NaiveDate::parse_from_str(&rows[6], "%Y/%m/%d").unwrap(),
            name: rows[7].clone(),
        };
        books.push(book);
    });

    Ok(books)
}
