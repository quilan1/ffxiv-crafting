# API overview

When connecting to the server's websocket, a stream of messages are sent to/from the server. The general flow of information is as follows:

* **Note:** Be sure to consult [The Code](frontend/src/app/(universalis)/universalis-api.ts) in case this is somewhat stale.
* A [Request](#request) object is sent from client to server.
* The server will send back a [Recipe](#recipe-message) message with the FFXIV database item information for the request.
* The server will periodically send [Status](#status-message) messages for all of the fetch requests the server has made to Universalis.
* As fetch requests complete (or fail), the server will send [Success](#success-message) or [Failure](#failure-message) messages, respectively.
* Once all fetch requests have either failed or succeeded, the server will send a [Done](#done-message) message and terminate the web socket.

## Request

Specifies the items & methodology for the client's item request.

| Name | Type | Default | Description |
|-|-|-|-|
| **query** | string | | The query string for items that are to be analyzed. See the [query language](README.md#query-language) format below for details. |
| **purchaseFrom** | string | | The world from where ingredients will be purchased. Additionally, a region (e.g. 'North America') or a data center (e.g. 'Dynamis') may be used. |
| **sellTo** | string | | The world, to where goods will be sold. |
| **retainNumDays** | number | 7.0 | Number of days to retain records for statistics. |
| **isCompressed** | boolean | false | Whether or not to gzip the remaining messages sent back & forth to the server. |

## Messages

Messages sent from the server to the client, will be one of the following objects:

| Name | Type | Description |
|-|-|-|
| **recipe** | [Recipe](#recipe-message) | Recipe information for the request. |
| **status** | [Status](#status-message) | Status information from the server for the current request. |
| **success** | [Success](#success-message) | Information for a portion of items has been successfully returned. |
| **failure** | [Failure](#failure-message) | Information for a portion of items that failed to fetch from Universalis. |
 **done** | [Done](#done-message) | The server is finished processing and will be terminating the websocket. |

### Recipe Message

* Recipe messages will be formatted: { 'recipe': [RecipeInfo](#recipeinfo-object) }.

#### RecipeInfo Object

| Name | Type | Default | Description |
|-|-|-|-|
| **itemInfo** | { \[integer\]: [ItemInfo](#iteminfo-object)} | | Record of [ItemInfos](#iteminfo-object) indexed by their `itemIds`, for *all* items involved in the query. |
| **topIds** | \[integer\] | | Array of item IDs for *only* the items that match the query; consumed & included ingredient IDs are not included. |

#### ItemInfo Object

| Name | Type | Default | Description |
|-|-|-|-|
| **itemId** | integer | | The ID of an item in the FFXIV database. |
| **name** | string | | Item IDs for *only* the items that match the query. Ingredient IDs are not included. |
| **recipe** | [Recipe](#recipe-object) | undefined | The recipe input & output information for crafting the item. |

#### Recipe Object

| Name | Type | Default | Description |
|-|-|-|-|
| **inputs** | \[[Ingredient](#ingredient-object)\] | | Array of crafting inputs required to make the recipe. |
| **outputs** | integer | | The number of items created when crafted. |
| **level** | integer | | The crafting level of the recipe. |

#### Ingredient Object

| Name | Type | Default | Description |
|-|-|-|-|
| **itemId** | integer | | ID of the ingredient used in crafting. |
| **count** | integer | | Number of items consumed when crafting. |

### Status Message

* Status messages will be formatted: { 'status': [Status](#status-object) }.

#### Status Object

This may be exactly one of the following:

* **Note**: Items that are quoted literals, e.g. 'active', are strings, not structured objects.

| Name | Type | Description |
|-|-|-|
| **'active'** | 'active' | The fetch request is currently being fetched from Universalis. |
| **'warn'** | 'warn' | The fetch request is currently being fetched from Universalis and has failed at least once already. |
| **finished** | boolean | If true, the request was successful, if false the information was unable to be fetched from Universalis. |
| **queued** | integer | The request is currently queued and will be processed in order. |

### Success Message

* Success messages will be formatted: { 'success': [Success](#success-object) }.

#### Success Object

| Name | Type | Description |
|-|-|-|
| **listings** | { \[integer\], \[Listing\] } | The listings currently on the market board. These are what are currently available. |
| **history** | { \[integer\], \[Listing\] } | The historical sales results for the market board. These are what have been sold in the past. |

#### Listing Object

Listings are used for both current market listings and prior sales of an item.

| Name | Type | Optional | Description |
|-|-|-|-|
| **price** | integer | | The listed gil cost of the listing or sale. |
| **count** | integer | | The stack size of the item for a listing or sale. |
| **isHq** | boolean | | The listing or sale is of a high quality item. |
| **daysSince** | number | | Number of days since a listing was placed, or an item was sold. |
| **world** | integer | *listings only* | What world the item is listed. |
| **name** | string | *listings only* | The name of the retainer selling an item. |

### Failure Message

* Failure messages will be formatted: { 'failure': \[integer\] }, detailed below.

| Type | Description |
|-|-|
| \[integer\] | IDs that failed being fetched from Universalis. |

### Done Message

* Done messages will be the literal string: 'done'.
