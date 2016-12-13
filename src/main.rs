extern crate slackbot;
extern crate bearbot;
extern crate regex;
extern crate dotenv;
extern crate iron;

use slackbot::{SlackBot, Sender};
use bearbot::handlers;

use dotenv::dotenv;
use std::env;
use std::thread;

use iron::prelude::*;
use iron::status;

fn main() {
    dotenv().ok();

    let username = env::var("USERNAME").unwrap();
    let token    = env::var("SLACK_API_TOKEN").unwrap();
    let mut bot = SlackBot::new(username.to_owned(), token);
    bot.on(r"login (?P<email>[^\s]*) (?P<password>.*)", Box::new(handlers::SessionHandler));
    bot.on(r"search (?P<keywords>.*)", Box::new(handlers::FindTalentsHandler));
    bot.on(format!(r"(hi|hey|hello|hallo) {}", username), Box::new(|sender: &mut Sender, _: &regex::Captures| {
        sender.respond_in_channel("Hey <3").unwrap();
    }));

    let host = env::var("HTTP_HOST");
    let port = env::var("HTTP_PORT").or(env::var("PORT"));

    if host.is_ok() && port.is_ok() {
        thread::spawn(move || {
            let host = format!("{}:{}", host.unwrap(), port.unwrap());

            Iron::new(|_: &mut Request| {
                Ok(Response::with((status::Ok, "Hello world!")))
            }).http(&*host).unwrap();
        });
    }

    bot.run().unwrap();
}
