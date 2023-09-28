use std::collections::HashMap;

use itertools::Itertools;

type FilterOptions = Vec<String>;

#[derive(Clone)]
pub struct Filter {
    pub tag: String,
    pub options: FilterOptions,
}

type FilterFn = for<'a> fn(&'a [String]) -> Option<(String, Vec<String>)>;

impl Filter {
    pub fn apply_filter_str(filter_str: &str) -> (String, Vec<String>) {
        if filter_str.trim().is_empty() {
            return (String::new(), Vec::new());
        }

        let mut db_filters = Vec::new();
        let filters = Self::from_str(filter_str);
        let filter_functions = Self::filter_functions();
        for Filter { tag, options } in filters {
            db_filters.push(match filter_functions.get(&tag[..]) {
                Some(func) => func(&options),
                None => {
                    if tag.chars().nth(0).unwrap_or(' ') == ':' {
                        log::info!(target: "ffxiv_items", "Invalid filter tag: {tag}");
                        continue;
                    }
                    log::info!(target: "ffxiv_items", "Missing filter tag: {tag}, interpreting it as a :name filter",);
                    let mut new_options = options.clone();
                    if let Some(option) = new_options.first_mut() {
                        *option = format!("{tag} {option}");
                    } else {
                        new_options.push(tag);
                    }
                    filter_name(&new_options)
                }
            })
        }

        let (sql_clauses, binds): (Vec<_>, Vec<_>) = db_filters.into_iter().flatten().unzip();
        let binds = binds.into_iter().flatten().collect_vec();
        (sql_clauses.join(" AND "), binds)
    }

    fn from_str(filter_str: &str) -> Vec<Filter> {
        filter_str
            .split(',')
            .map(|filter| {
                let filter = filter.trim();
                let contents = filter.split(' ').collect::<Vec<_>>();
                let (ftype, options) = if contents.len() > 1 {
                    (
                        contents[0].to_string(),
                        contents[1..]
                            .join(" ")
                            .split('|')
                            .map(str::trim)
                            .filter(|filter| !filter.is_empty())
                            .map(ToString::to_string)
                            .collect::<Vec<_>>(),
                    )
                } else {
                    (contents[0].to_string(), Vec::new())
                };
                Filter {
                    tag: ftype,
                    options,
                }
            })
            .collect()
    }

    fn filter_functions() -> HashMap<&'static str, FilterFn> {
        let mut map: HashMap<_, FilterFn> = HashMap::new();

        // Source filters
        // TODO: Prioritize this first
        // map.insert(":leve", filter_leve);

        // Normal filters
        map.insert(":name", filter_name);
        map.insert(":rlevel", filter_recipe_level);
        map.insert(":elevel", filter_equip_level);
        map.insert(":ilevel", filter_ilevel);
        map.insert(":cat", filter_ui_category);
        map.insert(":contains", filter_contains);
        map.insert(":includes", filter_includes);

        // Result filters
        map.insert(":count", filter_noop);
        map.insert(":limit", filter_noop);
        map.insert(":min_velocity", filter_noop);

        map
    }
}

////////////////////////////////////////////////////////////

fn filter_generic_regex(table_name: &str, options: &[String]) -> Option<(String, Vec<String>)> {
    if options.is_empty() {
        return None;
    }

    fn resp(table_name: &str, op: &str, pattern: String) -> Option<(String, Vec<String>)> {
        Some((format!("{table_name} {op} ?"), vec![pattern]))
    }

    let table_name = format!("{table_name}.name");
    let pattern = options.join("|");
    if let Some(postfix) = pattern.strip_prefix('!') {
        return resp(&table_name, "=", postfix.into());
    }

    if "([.*+$^".chars().any(|ch| pattern.contains(ch)) {
        return resp(&table_name, "RLIKE", pattern.replace(' ', "\\s"));
    }

    resp(&table_name, "LIKE", format!("%{pattern}%"))
}

fn filter_generic_range(field: &str, options: &[String]) -> Option<(String, Vec<String>)> {
    if options.is_empty() {
        return None;
    }

    let values = options
        .iter()
        .filter_map(|level| level.parse::<u32>().ok())
        .collect::<Vec<_>>();

    let min = values.first().cloned().unwrap_or(0);
    let max = values.last().cloned().unwrap_or(u16::MAX as u32);
    match values.len() {
        0 => None,
        1 => Some((format!("{field} = {min}"), Vec::new())),
        _ => Some((format!("{field} >= {min} AND {field} <= {max}"), Vec::new())),
    }
}

////////////////////////////////////////////////////////////

fn filter_name(options: &[String]) -> Option<(String, Vec<String>)> {
    filter_generic_regex("i", options)
}

fn filter_recipe_level(options: &[String]) -> Option<(String, Vec<String>)> {
    filter_generic_range("r.level", options)
}

fn filter_equip_level(options: &[String]) -> Option<(String, Vec<String>)> {
    filter_generic_range("i.equip_level", options)
}

fn filter_ilevel(options: &[String]) -> Option<(String, Vec<String>)> {
    filter_generic_range("i.item_level", options)
}

fn filter_ui_category(options: &[String]) -> Option<(String, Vec<String>)> {
    if options.is_empty() {
        return None;
    }

    let clause = options.iter().map(|_| "?").join(", ");
    let clause = format!("c.name IN ({clause})");
    Some((clause, options.to_vec()))
}

fn filter_contains(options: &[String]) -> Option<(String, Vec<String>)> {
    filter_generic_regex("i_g", options)
}

fn filter_includes(options: &[String]) -> Option<(String, Vec<String>)> {
    filter_generic_regex("i_n", options)
}

/*
fn filter_leve<'a>(options: &[String]) {
    let categories = options;
    let all_leve_items = library.all_leves.all_item_ids();

    items.retain(|item| all_leve_items.contains(&item.id));

    if options.is_empty() {
        return;
    }

    items.retain(|item| {
        let leve_ids = library.all_leves.get_by_item_id(item.id).unwrap();
        leve_ids
            .iter()
            .map(|leve_id| &library.all_leves[leve_id].jobs)
            .any(|jobs| library.all_job_categories[jobs].matches_any(categories))
    });
}
*/

#[allow(clippy::ptr_arg)]
fn filter_noop(_options: &[String]) -> Option<(String, Vec<String>)> {
    None
}

////////////////////////////////////////////////////////////
