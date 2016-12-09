#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate slackbot;
extern crate hyper;
extern crate url;
extern crate regex;

#[macro_use]
extern crate lazy_static;

pub mod honeypot {
    use std::io::{self, Read};
    use std::borrow::Borrow;
    use serde;
    use serde_json;
    use hyper;
    use hyper::client::Response;
    use hyper::header::{Headers, ContentType, Authorization};
    use hyper::mime::{Mime, TopLevel, SubLevel, Attr, Value};
    use url::{Url, form_urlencoded};

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct User {
        pub id:        u32,
        pub email:     String,
        pub firstname: String,
        pub lastname:  String
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct RecruiterSessionInfo {
        pub user:  User,
        pub role:  String,
            token: String
    }

    #[derive(Debug)]
    pub struct RecruiterSession {
        pub info:   RecruiterSessionInfo,
        pub client: Client
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct Talent {
        pub id:       u64,
        pub headline: Option<String>
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct Meta {
        pub total: u64
    }

    #[derive(Debug, PartialEq, Deserialize)]
    pub struct FoundTalents {
        pub talents: Vec<Talent>,
        pub meta:    Meta
    }

    #[derive(Debug)]
    pub enum ClientError {
        HTTPError(hyper::Error),
        IOError(io::Error),
        JSONError(serde_json::Error)
    }

    #[derive(Debug)]
    pub struct Client {
        pub base_url: String,
            headers:  Headers
    }

    impl Client {
        pub fn post<I, K, V, T>(&self, endpoint: &str, pairs: I) -> Result<T, ClientError>
            where I: IntoIterator, I::Item: Borrow<(K, V)>, K: AsRef<str>, V: AsRef<str>, T: serde::Deserialize
        {
            let mut response = hyper::Client::new()
                                    .post(&*format!("{}{}", self.base_url, endpoint))
                                    .headers(self.headers.to_owned())
                                    .body(&*form_urlencoded::Serializer::new(String::new())
                                          .extend_pairs(pairs)
                                          .finish())
                                    .send();

            self.parse_response(response)
        }

        pub fn get<I, K, V, T>(&self, endpoint: &str, pairs: I) -> Result<T, ClientError>
            where I: IntoIterator, I::Item: Borrow<(K, V)>, K: AsRef<str>, V: AsRef<str>, T: serde::Deserialize
        {
            let mut url = Url::parse(&*format!("{}{}", self.base_url, endpoint)).unwrap();
            url.query_pairs_mut().clear().extend_pairs(pairs);

            let mut response = hyper::Client::new()
                                    .get(url)
                                    .headers(self.headers.to_owned())
                                    .send();

            self.parse_response(response)
        }


        pub fn parse_response<T>(&self, response: Result<Response, hyper::Error>) -> Result<T, ClientError> where T: serde::Deserialize {
            match response {
                Ok(mut response) => {
                    let mut body = String::new();
                    match response.read_to_string(&mut body) {
                        Ok(_)  => {},
                        Err(e) => { return Err(ClientError::IOError(e)) },
                    }

                    match serde_json::from_str(&body) {
                        Ok(r)  => Ok(r),
                        Err(e) => Err(ClientError::JSONError(e))
                    }
                },
                Err(e) => Err(ClientError::HTTPError(e))
            }
        }
    }

    impl RecruiterSession {
        /// Sign in
        pub fn new(url: &str, email: &str, password: &str) -> Result<RecruiterSession, ClientError> {
            let mut headers = Headers::new();
            headers.set(
                ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded,
                     vec![(Attr::Charset, Value::Utf8)])));

            let client     = Client { base_url: url.to_owned(), headers: headers.to_owned() };
            let login_data = vec![
                ("user[email]", email),
                ("user[password]", password)
            ];
            let recruiter: RecruiterSessionInfo = try!(client.post("/api/v1/users/login", login_data.into_iter()));

            headers.set(
                Authorization(format!("Token {}", recruiter.token)));

            Ok(RecruiterSession {
                client: Client { base_url: url.to_owned(), headers: headers },
                info:   recruiter
            })
        }

        pub fn find_talents(&self, keywords: &str) -> Result<FoundTalents, ClientError> {
            let params = vec![
                ("keywords", keywords)
            ];
            self.client.get("/api/v1/company/talents", params.into_iter())
        }
    }
}

pub mod handlers {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use regex::{self, Regex};
    use slackbot::{SlackBot, Sender, CommandHandler};
    use honeypot;

    lazy_static! {
        static ref SESSIONS: Mutex<HashMap<String, honeypot::RecruiterSession>> = Mutex::new(HashMap::new());
        static ref MAILTO_RE: Regex = Regex::new(r"<mailto:(?P<email>[^\|]*)").unwrap();
    }

    pub struct SessionHandler;
    impl CommandHandler for SessionHandler {
        fn handle(&mut self, sender: &mut Sender, args: &regex::Captures) {
            let email = MAILTO_RE
                .captures(args.name("email").unwrap()).unwrap()
                .name("email").unwrap();
            let password = args.name("password").unwrap();
            let session = honeypot::RecruiterSession::new("https://staging-app.honeypot.co", email, password).unwrap();
            sender.respond_in_channel(format!("Hello {}!", session.info.user.firstname));
            SESSIONS.lock().unwrap().insert(sender.user.id.to_owned(), session);
        }
    }

    pub struct FindTalentsHandler;
    impl CommandHandler for FindTalentsHandler {
        fn handle(&mut self, sender: &mut Sender, args: &regex::Captures) {
            let sessions = SESSIONS.lock().unwrap();
            match sessions.get(&sender.user.id) {
                Some(session) => {
                    let results = session.find_talents(&args.name("keywords").unwrap());
                    sender.respond_in_channel(results.unwrap().talents.iter()
                                              .map(|t| {
                                                  let url = format!("{}/company/talents/{}", session.client.base_url, t.id);
                                                  match t.headline.to_owned() {
                                                      Some(headline) => format!("{}\n{}\n\n", headline, url),
                                                      None           => format!("{}\n\n", url)
                                                  }
                                              })
                                              .collect::<Vec<String>>().join("\n"));
                },
                None => { sender.respond_in_channel("I can't do this if you don't sign in as recruiter :("); }
            };
        }
    }
}