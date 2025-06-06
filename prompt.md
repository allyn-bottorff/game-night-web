# Instructions
Create a web site for creating and hosting polls for "Platform Engineering Game
Night". There will be a frontend and a backend with the details each in their
own section.

## Purpose
The website will host polls for options for game nights. Platform Engineering
members will log into the website.

### Features
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

## Frontend
The frontend will be HTML5, CSS, and vanilla javascript.

### Frontend Features
* Support all of the features as described in the previous "Purpose" section.
* Use modern style, but prefer density.
* Users should be able to interact with polls from their phones.
* Use modern security best practices.
* The frontend will interact with the backend via REST calls and HTTP.

## Backend
The backend will be a Rust application using SQLite for a database.

### Backend Features
* Support all of the features as described in the previous "Purpose" section.
* Prefer native rust over crates where possible.
* Will run as a single instance with SQLite for the database.
* Use Axum as the API framework
* Use Tokio for async support.
* Create a prometheus-compatible metrics endpoint
* Use the env_logger crate for standard text logging
