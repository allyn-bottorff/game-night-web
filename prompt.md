# Instructions
Create a web site for creating and hosting polls for "Platform Engineering Game
Night". There will be a frontend and a backend with the details each in their
own section.

## Purpose
The website will host polls for options for game nights. Platform Engineering
members will log into the website.

### Features
* The application will be build using Rust and the Rocket framework.
* Use the latest version of packages.
* Platform Engineering members will be able to log into the website
* Members will be able to create new polls
  * Polls will be able to have arbitrary options
  * Polls will be able to have expiration dates
  * Active polls will be in their own section of the frontend web page
  * Inactive (or expired) polls will be able to be viewed in their own section
* There will be no anonymous access to the website
* Members will be allowed to add new members.
* Poll results will be able to be viewed live. Members do not need to wait until
  the poll is closed to see current results.
* Support calendar dates and times as poll options
* Prefer native rust over crates where possible.
* Will run as a single instance with SQLite for the database.
* Create a prometheus-compatible metrics endpoint
