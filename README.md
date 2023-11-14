# FFXIV Crafting Marketboard

## Overview

This program is designed as a custom interface to Universalis, and the market information contained within. The general flow of information, is that the user uses the website to generate a query string, which is sent to the rust server. This server takes the requests, determines which IDs the query-string represents, calls universalis, grabs the history & listing info, packages it together, and sends it back to the frontend. The data is then grouped & correlated together into a comprehensive profit / loss table, where the purchase information for items is displayed.

## Setup

The only real requirement is Docker Compose. For windows, this may be found by installing Docker Desktop. Once installed, hosting the server & frontend may be done with the command (in this repo):

> docker-compose up -d

This will take some time for the rust server to compile the project (running tests, building the final product), and for the frontend to setup.

### Example usage

To access the webpage, simply navigate on your browser to:
[http://localhost](http://localhost).

As an example of usage, where the 'Filters:' dropdown sits, choose 'Level 90 Crafting Mats' as an example, and click 'Refresh'. This should, after a brief communication, fill the table with crafting information.

## Project Organization

The Rust crates are inside of the `crates` directory, and the Typescript files are inside the `frontend` directory. The general flow of information is as follows:

### Frontend

* [frontend](frontend): A Next.JS React project that handles all of the statistics calculations & layout of the information.

### Crates

* [ffxiv_server](crates/ffxiv_server): The main entry point for the rust code. Consumes `ffxiv_items` to turn a query string (e.g. ":name Item Name") into a series of IDs.
* [ffxiv_universalis](crates/ffxiv_universalis): Controls all of the interactions with the universalis website. Uses `async_processor` to ensure that all server requests are funneled through a pipe that executes no more than 8 requests concurrently.
* [ffxiv_items](crates/ffxiv_items): Functions as an item info singleton repository.
* [mock_traits](crates/mock_traits): Some simple traits & implementations for things that may be mocked, such as downloading a file from a website.

## NGINX

* [nginx](nginx/nginx.conf): For the docker container, a simple NGINX setup is used to internally connect the three internal servers.

## Brief API overview

* GET `ws://{SERVER_ADDR}/v1/universalis`: Connects to a websocket that transmits data about the universalis request. Further details may be found [here](docs/api.md).

## Query Format

The query format is documented [here](docs/query-format.md).
