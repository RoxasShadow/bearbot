extern crate slackbot;
extern crate bearbot;
extern crate regex;
extern crate dotenv;

use slackbot::{SlackBot, Sender};
use bearbot::handlers;

use dotenv::dotenv;
use std::env;

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

    bot.run().unwrap();
}
