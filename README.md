# FFXIV Crafting Marketboard

## Overview

This program is designed as a custom interface to Universalis, and the market information contained within. The general flow of information, is that the user uses the website to generate a query string, which is sent to the rust server. This server takes the requests, determines which IDs the query-string represents, calls universalis, grabs the history & listing info, packages it together, and sends it back to the frontend. The data is then grouped & correlated together into a comprehensive profit/loss table, where the purchase information for items is displayed.

## Setup

The only real requirement is Docker Compose. For windows, this may be found by installing Docker Desktop. Once installed, hosting the server & frontend may be done with the command (in this repo):

> docker-compose up -d

This will take some time for the rust server to compile the project for the first time (first in debug to run tests, then in release), but once the server is healthy, the http service will spin up. This will fire up an apache server on port 8080 for the frontend, and the rust server on port 3001.

### Example usage

To access the webpage, simply navigate on your browser to:
[http://localhost:8080](http://localhost:8080).

As an example of usage, where the 'Filters:' dropdown sits, choose 'Level 90 Crafting Mats' as an example, and click 'Refresh'. This should, after a brief communication, fill the table with crafting information.

## Project Organization

The Rust crates are inside of the 'crates' directory, and the Javascript / Typescript files are inside the 'frontend' directory. The general flow of information is as follows:

### Frontend

* [src-ts](frontend/src-ts): The typescript is compiled by the docker 'typescript-init' service, into the `src-js/js` directory. Calls `ffxiv_server` API routines when the Refresh / Cancel button is pressed.
* [src-js](frontend/src-js): This houses the index.html and compiled typescript code.

### Backend

* [ffxiv_server](crates/ffxiv_server): The main entry point for the rust code. Consumes `ffxiv_items` to turn a filter string (e.g. ":name Item Name") into a series of IDs.
* [ffxiv_universalis](crates/ffxiv_universalis): Controls all of the interactions with the universalis website. Uses `async_processor` to ensure that all server requests are funneled through a pipe that executes no more than 8 requests concurrently.
* [ffxiv_items](crates/ffxiv_items): Functions as an item info singleton repository.
* [async_processor](crates/async_processor): Simple structure that executes futures concurrently, with at most `max_active` executed at a time.

## Brief API overview

* GET `ws://{SERVER_ADDR}/v1/universalis`: Connects to a websocket that transmits data about the universalis request

## Customization

This is all a personal project, and not intended for anyone else, but in the off-chance that someone wants to use it, references to the homeworld (Seraph) may be found in [frontend/src-ts/src/custom/statistics.ts](frontend/src-ts/src/custom/statistics.ts), as the HOMEWORLD variable.
