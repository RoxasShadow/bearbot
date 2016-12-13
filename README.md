Bearbot
=======
A WIP bot for [Honeypot](https://www.honeypot.io).

How to use
----------
`$ cp .env.example .env`

[Create a new bot](http://slack.com/services/new/bot) for your Slack account
and save the token in the `.env` file as `SLACK_API_TOKEN` (`$ cp .env.example .env` may help).

If you want to run it on Heroku, remember to move the `.env` variables to Heroku's.
Also, in case you use a free plan, you should setup an [UptimeRobot](https://uptimerobot.com)
to avoid that Heroku puts your application to sleep.
