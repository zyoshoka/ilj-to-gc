use chrono::{Duration, NaiveDate, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{book::{Book, RESERVED}, Config};

#[derive(Serialize)]
struct Claims {
    iss: String,
    scope: String,
    aud: String,
    exp: i64,
    iat: i64,
}

#[derive(Serialize)]
struct Body {
    assertion: String,
    grant_type: String,
}

#[derive(Deserialize)]
struct Res {
    access_token: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Date {
    pub date: NaiveDate,
}

#[derive(Debug, Deserialize, Serialize)]
struct Event {
    kind: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    etag: Option<String>,

    id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,

    start: Date,
    end: Date,
}

impl Event {
    pub fn from_book(book: &Book) -> Self {
        Event {
            kind: "calendar#event".into(),
            etag: None,
            id: book.get_id(),
            summary: Some(book.get_summary()),
            start: book.get_start_date(),
            end: book.get_end_date(),
        }
    }

    pub fn set_etag_from_event(&mut self, event: &Event) {
        self.etag = Some(event.etag.clone().unwrap().trim_matches('"').into());
    }

    pub fn is_up_to_date_with(&self, book: &Book) -> bool {
        self.end == book.get_end_date() && self.is_marked_as_reserved() == book.is_reserved
    }

    fn is_marked_as_reserved(&self) -> bool {
        self.summary.clone().unwrap().contains(RESERVED)
    }
}

#[derive(Debug, Deserialize)]
struct Events {
    pub items: Vec<Event>,
}

pub async fn subscribe_to_calender(client: &reqwest::Client, config: &Config, books: &Vec<Book>) {
    let calendar_event_base_url = "https://www.googleapis.com/calendar/v3/calendars/".to_string() + &config.calendar_id;
    let access_token = get_access_token(client, config).await;

    let events = get_events(client, calendar_event_base_url.clone(), access_token.clone()).await;

    for book in books {
        let mut same_exists = false;
        for event in &events {
            if event.id == book.get_id() {
                same_exists = true;
                if !event.is_up_to_date_with(book) {
                    println!("{:?}", event);
                    update_event(client, calendar_event_base_url.clone(), access_token.clone(), book, event).await;
                }
                break;
            }
        }
        if same_exists {
            continue;
        }

        create_event(client, calendar_event_base_url.clone(), access_token.clone(), book).await;
    }
}

async fn get_access_token(client: &reqwest::Client, config: &Config) -> String {
    let header = Header {
        typ: Some("JWT".into()),
        alg: Algorithm::RS256,
        kid: Some(config.google_private_key_id.clone()),
        ..Default::default()
    };
    let claims = Claims {
        iss: config.google_client_email.clone(),
        scope: "https://www.googleapis.com/auth/calendar.events".into(),
        aud: "https://www.googleapis.com/oauth2/v3/token".into(),
        iat: Utc::now().timestamp(),
        exp: (Utc::now() + Duration::minutes(10)).timestamp(),
    };
    let key = EncodingKey::from_rsa_pem(config.google_private_key.as_bytes()).unwrap();
    let token = encode(&header, &claims, &key);
    let body = Body {
        assertion: token.unwrap(),
        grant_type: "urn:ietf:params:oauth:grant-type:jwt-bearer".into(),
    };

    client.post("https://www.googleapis.com/oauth2/v3/token")
        .form(&body)
        .send().await.unwrap()
        .json::<Res>().await.unwrap().access_token
}

async fn get_events(client: &reqwest::Client, calendar_event_base_url: String, access_token: String) -> Vec<Event> {
    client.get(calendar_event_base_url.clone() + "/events")
        .bearer_auth(access_token.clone())
        .send().await.unwrap()
        .json::<Events>().await.unwrap().items
}

async fn create_event(client: &reqwest::Client, calendar_event_base_url: String, access_token: String, book: &Book) {
    let event = Event::from_book(book);

    client.post(calendar_event_base_url.clone() + "/events")
        .bearer_auth(access_token.clone())
        .json(&event)
        .send().await.unwrap();
}

async fn update_event(client: &reqwest::Client, calendar_event_base_url: String, access_token: String, book: &Book, event: &Event) {
    let mut updated_event = Event::from_book(book);
    updated_event.set_etag_from_event(event);

    client.put(calendar_event_base_url.clone() + "/events/" + &book.get_id())
        .bearer_auth(access_token.clone())
        .json(&updated_event)
        .send().await.unwrap();
}
