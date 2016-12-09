extern crate slackbot;
extern crate bearbot;
extern crate regex;

use slackbot::{SlackBot, Sender};
use bearbot::handlers;

fn main() {
    let mut bot = SlackBot::new("bearbot", "token");
    bot.on(r"login (?P<email>[^\s]*) (?P<password>\w*)", Box::new(handlers::SessionHandler));
    bot.on(r"search (?P<keywords>.*)", Box::new(handlers::FindTalentsHandler));
    bot.on(r"(hi|hey|hello|hallo) bearbot", Box::new(|sender: &mut Sender, _: &regex::Captures| {
        sender.respond_in_channel("Hey <3").unwrap();
    }));

    bot.run().unwrap();
}
