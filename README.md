# FFXIV Crafting Marketboard

## Overview

This program is designed as a custom interface to Universalis, and the market information contained within.
The general flow of information, is that the user uses the website to generate a query string, which is sent
to the rust server. This server takes the requests, determines which IDs the query-string represents, calls
universalis, grabs the history & listing info, packages it together, and sends it back to the frontend. The
data is then grouped & correlated together into a comprehensive profit/loss table, where the purchase information
for items is displayed.

## Setup

The only real requirement is Docker Compose. For windows, this may be found by installing Docker Desktop. Once
installed, hosting the server & frontend may be done with the command (in this repo):

> docker-compose up -d

This will take some time for the rust server to compile the project for the first time, but once the server is
healthy, the http service will spin up. This will fire up an apache server on port 8080 for the frontend, and
the rust server on port 3001. To access the webpage, simply navigate on your browser to:
[http://localhost:8080](http://localhost:8080).

Where the 'Filters:' dropdown sits, choose 'Level 90 Crafting Mats' as an example, and click 'Refresh'. This
should, after a brief communication, fill the table with crafting information.

## Project Organization

The Rust crates are inside of the 'crates' directory, and the Javascript / Typescript files are inside the
'frontend' directory.

## Customization

This is all a personal project, and not intended for anyone else, but in the off-chance that someone wants to
use it, references to the homeworld (Seraph) may be found in
[crates/ffxiv_server/src/cli.rs](crates/ffxiv_server/src/cli.rs)
