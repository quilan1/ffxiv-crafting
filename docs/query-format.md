# Query Language

## General Structure

* **Queries**: Queries consist of a set of clauses that are inclusively joined (boolean OR). Each clause is separated by a semicolon, e.g: `<clause #1>; <clause #2>; <...>`. The results of each individual clause are merged into the returned items.
* **Clause**: A clause consists of several filters that are joined via a boolean AND operation. Each filter is separated by a comma, e.g. `<filter #1>, <filter #2>, <...>`. Each successive filter narrows the results of the final set of returned items.
* **Filter**: A filter consists of a tag, typically followed by pipe (|) delimited options.

## Filter Tags Reference

### :name

* `:name <item-name>`: Matches on the name of the item. An example filter of this is: `:name Iron`, which will return items that contain iron somewhere inside the name.
  * `:name !<exact-item-name>`: Matches the exact name, and nothing more. It must be the whole & complete name of the item.
  * `:name <regexp>`: Matches a regex phrase for the name. Useful for complex queries.
  * **Examples**:
    * `:name Persimmon Pudding`, returns both '*Persimmon Pudding*' and '*Rarefied Persimmon Pudding*'.
    * `:name !Persimmon Pudding`, returns only '*Persimmon Pudding*'.
    * `:name Timeworn [O|K].*skin map`, returns '*Timeworn Ophiotauroskin Map*' and '*Timeworn Kumbhiraskin Map*'.
  * **Note:** All name matches are caseless.

### :rlevel

* `:rlevel <min-level>|<max-level>`: Matches on items with a recipe in the level range. Also accepts a single-argument version for an exact level match.
  * `:name ^Rarefied, :rlevel 61|69`, returns the crafting scrip recipes for level 61 to 69, inclusive.

### :elevel

* `:elevel <min-level>|<max-level>`: Matches on a character's level to wear/equip (1-90), not item level (1-~650). May change its name soon. Also accepts a single-argument version for an exact level match.
  * `:elevel 90, :name Voidvessel`, returns all of the level 90 Voidvessel gear.

### :ilevel

* `:ilevel <min-level>|<max-level>`: Matches on an item's item level (1-~665), not character level (1-90). Also accepts a single-argument version for an exact level match.
  * `:ilevel 655, :name Voidvessel`, returns, same as above, all of the level 90 Voidvessel gear.

### :cat

* `:cat <category #1>|<category #2>|<...>`: Matches on an item's category, as presented in the UI. May also take the '!' prefix for exact matches or regex.
  * `:rlevel 90, :cat !Metal|Lumber`, returns the metal and lumber crafted items.

### :contains

* `:contains <name>`, first matches of an item's name, searches for items with the former as a primary ingredient in their recipe. May also take the '!' prefix for exact matches or regex.
  * `:contains eagle feather`, returns a number of items that use '*Eagle Feather*' as a primary ingredient, e.g. '*Blessed Fletchings*'.

### :includes

* `:includes <name>`, like above, but the name matched item may live anywhere down the recipe ingredient chain. May also take the '!' prefix for exact matches or regex.
  * `:includes !maple branch`, returns items that have '*Maple Branch*' anywhere down the recipe ingredient chain, e.g '*Budding Maple Wand*', which requires '*Maple Wand*', which requires '*Maple Branch*'
