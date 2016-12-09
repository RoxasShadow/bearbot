#
#            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
#                    Version 2, December 2004
#
# Everyone is permitted to copy and distribute verbatim or modified
# copies of this license document, and changing it is allowed as long
# as the name is changed.
#
#            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
#   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
#
#  0. You just DO WHAT THE FUCK YOU WANT TO.
##
require 'dotenv'
Dotenv.load

require 'slack-ruby-bot'
require 'excon'
require 'json'
require 'pry'

module Honeypot
  class AuthenticationError < StandardError
    def message
      'Authentication failed.'
    end
  end

  class Recruiter
    attr_reader :url

    def initialize(url, email, password)
      @url        = url
      @connection = Excon.new(url)
      sign_in(email, password)
      @connection = Excon.new(url, headers: auth_headers)
    end

    def infos
      @user['user']
    end

    def find_talents(keywords)
      query    = { keywords: keywords }
      response = connection.get(path: '/api/v1/company/talents', query: query)
      JSON.parse(response.body)
    end

    private

    attr_reader :connection, :user

    def sign_in(email, password)
      response = connection.post({
        path: '/api/v1/users/login',
        body: URI.encode_www_form({
          'user[email]'    => email,
          'user[password]' => password
        }),
        headers: { 'Content-Type' => 'application/x-www-form-urlencoded' }
      })

      raise AuthenticationError if response.status != 201

      @user = JSON.parse(response.body)
    end

    def auth_headers
      { 'Authorization' => "Token #{user['token']}" }
    end
  end
end

$users = {}

class Bearbot < SlackRubyBot::Bot
  help do
    title 'Bearbot'
    desc 'yaaaay'

    command('login <email> <password>') do
      desc 'Sign you in as recruiter (private message!)'
    end

    command('search [keywords]') do
      desc 'Search talents for given keyword (i.e. "frontend developer")'
    end
  end

  match(/login (?<email>[^\s]*) (?<password>\w*)/i) do |client, data, match|
    begin
      email = URI.extract(match['email'])[0].split('mailto:')[1]
      user  = ($users[data['user']] = Honeypot::Recruiter.new(ENV['URL'], email, match['password']))

      client.say(text: "Hey #{user.infos['firstname']}", channel: data.channel)
    rescue Honeypot::AuthenticationError => e
      client.say(text: e.message, channel: data.channel)
    end
  end

  match(/search (?<keywords>.*)/i) do |client, data, match|
    if (user = $users[data['user']])
      talents = user.find_talents(match['keywords'])['talents']
        .map { |talent| "#{talent['headline']}\n#{user.url}/company/talents/#{talent['id']}" }
        .join("\n\n")
      client.say(text: talents, channel: data.channel)
    else
      client.say(text: "I can't do this if you don't sign in as recruiter :(", channel: data.channel)
    end
  end
end

Bearbot.run
