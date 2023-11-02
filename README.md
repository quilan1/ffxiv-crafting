# FFXIV Crafting Marketboard

## Overview

This program is designed as a custom interface to Universalis, and the market information contained within. The general flow of information, is that the user uses the website to generate a query string, which is sent to the rust server. This server takes the requests, determines which IDs the query-string represents, calls universalis, grabs the history & listing info, packages it together, and sends it back to the frontend. The data is then grouped & correlated together into a comprehensive profit/loss table, where the purchase information for items is displayed.

## Setup

The only real requirement is Docker Compose. For windows, this may be found by installing Docker Desktop. Once installed, hosting the server & frontend may be done with the command (in this repo):

> docker-compose up -d

This will take some time for the rust server to compile the project for the first time (first in debug to run tests, then in release). It will simultaneously compile the frontend code at the same time. Once they're both done, a React server will have spun up on port 3000 for the frontend, and the rust server on port 3001.

### Example usage

To access the webpage, simply navigate on your browser to:
[http://localhost](http://localhost).

As an example of usage, where the 'Filters:' dropdown sits, choose 'Level 90 Crafting Mats' as an example, and click 'Refresh'. This should, after a brief communication, fill the table with crafting information.

## Project Organization

The Rust crates are inside of the 'crates' directory, and the Typescript files are inside the 'frontend' directory. The general flow of information is as follows:

### Frontend

* [frontend](frontend): The frontend has been completely rewritten as a Next.JS React project. It's now hosted on port 3000, but routed through nginx to port 80.

### Backend

* [ffxiv_server](crates/ffxiv_server): The main entry point for the rust code. Consumes `ffxiv_items` to turn a query string (e.g. ":name Item Name") into a series of IDs.
* [ffxiv_universalis](crates/ffxiv_universalis): Controls all of the interactions with the universalis website. Uses `async_processor` to ensure that all server requests are funneled through a pipe that executes no more than 8 requests concurrently.
* [ffxiv_items](crates/ffxiv_items): Functions as an item info singleton repository.
* [mock_traits](crates/mock_traits): Some simple traits & implementations for things that may be mocked, such as downloading a file from a website.

## Brief API overview

* GET `ws://{SERVER_ADDR}/v1/universalis`: Connects to a websocket that transmits data about the universalis request

## Customization

This is all a personal project, and not intended for anyone else, but in the off-chance that someone wants to use it, references to the homeworld (Seraph) may be found in [frontend/src/app/(universalis)/statistics.ts](frontend/src/app/(universalis)/statistics.ts), as the HOMEWORLD variable.

## Query Language

* **Queries**: Queries consist of a set of clauses that are inclusively joined (boolean OR). Each clause is separated by a semicolon, e.g: `<clause #1>; <clause #2>; <...>`. The results of each individual clause are merged into the returned items.
* **Clause**: A clause consists of several filters that are joined via a boolean AND operation. Each filter is separated by a comma, e.g. `<filter #1>, <filter #2>, <...>`. Each successive filter narrows the results of the final set of returned items.
* **Filter**: A filter consists of a tag, typically followed by pipe (|) delimited options.

### Tags

* `:name <item-name>`: Matches on the name of the item. An example filter of this is: `:name Iron`, which will return items that contain iron somewhere inside the name.
  * `:name !<exact-item-name>`: Matches the exact name, and nothing more. It must be the whole & complete name of the item.
  * `:name <regexp>`: Matches a regex phrase for the name. Useful for complex queries.
  * **Examples**:
    * `:name Persimmon Pudding`, returns both '*Persimmon Pudding*' and '*Rarefied Persimmon Pudding*'.
    * `:name !Persimmon Pudding`, returns only '*Persimmon Pudding*'.
    * `:name Timeworn [O|K].*skin map`, returns '*Timeworn Ophiotauroskin Map*' and '*Timeworn Kumbhiraskin Map*'.
  * **Note:** All name matches are caseless.
* `:rlevel <min-level>|<max-level>`: Matches on items with a recipe in the level range. Also accepts a single-argument version for an exact level match.
  * `:name ^Rarefied, :rlevel 61|69`, returns the crafting scrip recipes for level 61 to 69, inclusive.
* `:elevel <min-level>|<max-level>`: Matches on a character's level to wear/equip (1-90), not item level (1-~650). May change its name soon. Also accepts a single-argument version for an exact level match.
  * `:elevel 90, :name Voidvessel`, returns all of the level 90 Voidvessel gear.
* `:ilevel <min-level>|<max-level>`: Matches on an item's item level (1-~665), not character level (1-90). Also accepts a single-argument version for an exact level match.
  * `:ilevel 655, :name Voidvessel`, returns, same as above, all of the level 90 Voidvessel gear.
* `:cat <category #1>|<category #2>|<...>`: Matches on an item's category, as presented in the UI. May also take the '!' prefix for exact matches or regex.
  * `:rlevel 90, :cat !Metal|Lumber`, returns the metal and lumber crafted items.
* `:contains <name>`, first matches of an item's name, searches for items with the former as a primary ingredient in their recipe. May also take the '!' prefix for exact matches or regex.
  * `:contains eagle feather`, returns a number of items that use '*Eagle Feather*' as a primary ingredient, e.g. '*Blessed Fletchings*'.
* `:includes <name>`, like above, but the name matched item may live anywhere down the recipe ingredient chain. May also take the '!' prefix for exact matches or regex.
  * `:includes !maple branch`, returns items that have '*Maple Branch*' anywhere down the recipe ingredient chain, e.g '*Budding Maple Wand*', which requires '*Maple Wand*', which requires '*Maple Branch*'
