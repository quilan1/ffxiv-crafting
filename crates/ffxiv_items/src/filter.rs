use std::collections::HashMap;

use itertools::Itertools;

use crate::tables::{IngredientTable, InputIdsTable, ItemInfoTable, RecipeTable, UiCategoryTable};

type FilterOptions = Vec<String>;

#[derive(Clone)]
pub struct Filter {
    pub tag: String,
    pub options: FilterOptions,
}

pub struct FilterBindingInfo {
    pub clause: String,
    pub binds: Vec<String>,
}

type FilterFn = for<'a> fn(&'a [String]) -> Option<FilterBindingInfo>;

impl Filter {
    pub(crate) fn from_filter_str(filter_str: &str) -> Option<FilterBindingInfo> {
        if filter_str.trim().is_empty() {
            return None;
        }

        FilterBindingInfo::join(
            " OR ",
            Self::parse_all_filters(filter_str)
                .into_iter()
                .map(Self::filter_group_clause),
        )
    }

    fn filter_group_clause(filter_group: Vec<Filter>) -> Option<FilterBindingInfo> {
        let mut db_filters = Vec::new();
        let filter_functions = Self::filter_functions();
        for Filter { tag, options } in filter_group {
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

        FilterBindingInfo::join(" AND ", db_filters.into_iter())
    }

    fn parse_all_filters(filter_str: &str) -> Vec<Vec<Filter>> {
        filter_str.split(';').map(Self::parse_filters).collect()
    }

    fn parse_filters(filter_str: &str) -> Vec<Filter> {
        let filters = filter_str.split(',').collect_vec();
        let mut merged_filters = Vec::new();
        let mut accumulated = "".to_string();
        for filter in filters {
            let skip = filter.ends_with('\\');
            let filter = if !skip {
                filter
            } else {
                &filter[..filter.len() - 1]
            };
            let cur = format!("{}{filter}", accumulated);
            accumulated = "".to_string();
            if !skip {
                merged_filters.push(cur);
            } else {
                accumulated = format!("{cur},");
            }
        }

        merged_filters
            .iter()
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

impl FilterBindingInfo {
    fn from_op(table_name: &str, op: &str, bind_str: &str, binds: Vec<String>) -> Self {
        Self {
            clause: format!("{table_name} {op} {bind_str}"),
            binds,
        }
    }

    fn join<I: Iterator<Item = Option<Self>>>(join_op: &str, iter: I) -> Option<Self> {
        let (clauses, binds): (Vec<_>, Vec<_>) =
            iter.flatten().map(|info| (info.clause, info.binds)).unzip();

        match clauses.is_empty() {
            true => None,
            false => Some(FilterBindingInfo {
                clause: clauses.join(join_op),
                binds: binds.into_iter().flatten().collect(),
            }),
        }
    }
}

////////////////////////////////////////////////////////////

enum StringCompareType<'a> {
    Exact(&'a str),
    Regexp(&'a str),
    Like(&'a str),
}

fn filter_generic_regex(table_name: &str, options: &[String]) -> Option<FilterBindingInfo> {
    if options.is_empty() {
        return None;
    }

    let table_name = format!("{table_name}.name");
    let pattern = options.join("|");
    Some(match regex_string_compare_type(&pattern) {
        StringCompareType::Exact(pattern) => {
            let binds = pattern.split('|').map(String::from).collect_vec();
            let bind_str = binds.iter().map(|_| "?").join(", ");
            let bind_str = format!("({bind_str})");
            FilterBindingInfo::from_op(&table_name, "IN", &bind_str, binds)
        }
        StringCompareType::Regexp(pattern) => {
            FilterBindingInfo::from_op(&table_name, "RLIKE", "?", vec![pattern.replace(' ', "\\s")])
        }
        StringCompareType::Like(pattern) => {
            FilterBindingInfo::from_op(&table_name, "LIKE", "?", vec![format!("%{pattern}%")])
        }
    })
}

fn regex_string_compare_type(pattern: &str) -> StringCompareType<'_> {
    // If the string begins with '!', it's an exact string match
    // If it is grouped with parentheses, strip them out first
    if let Some(pattern) = pattern.strip_prefix('!') {
        return if pattern.contains('|') && pattern.starts_with('(') || pattern.ends_with(')') {
            StringCompareType::Exact(&pattern[1..pattern.len() - 1])
        } else {
            StringCompareType::Exact(pattern)
        };
    }

    // If it has any regex characters, it's regex, else like
    if "([.*+$^|".chars().any(|ch| pattern.contains(ch)) {
        StringCompareType::Regexp(pattern)
    } else {
        StringCompareType::Like(pattern)
    }
}

fn filter_generic_range(field: &str, options: &[String]) -> Option<FilterBindingInfo> {
    if options.is_empty() {
        return None;
    }

    let values = options
        .iter()
        .filter_map(|level| level.parse::<u32>().ok())
        .collect::<Vec<_>>();

    let min = values.first().cloned().unwrap_or(0);
    let max = values.last().cloned().unwrap_or(u16::MAX as u32);
    let binds = Vec::new();
    match values.len() {
        0 => None,
        1 => Some(FilterBindingInfo {
            clause: format!("{field} = {min}"),
            binds,
        }),
        _ => Some(FilterBindingInfo {
            clause: format!("{field} >= {min} AND {field} <= {max}"),
            binds,
        }),
    }
}

////////////////////////////////////////////////////////////

fn filter_name(options: &[String]) -> Option<FilterBindingInfo> {
    filter_generic_regex("i", options)
}

fn filter_recipe_level(options: &[String]) -> Option<FilterBindingInfo> {
    let Some(FilterBindingInfo { clause, binds }) = filter_generic_range("r.level", options) else {
        return None;
    };

    Some(FilterBindingInfo {
        clause: format!(
            "i.id IN (
                SELECT r.id
                FROM {} AS r
                WHERE {}
            )",
            RecipeTable::SQL_TABLE_NAME,
            clause
        ),
        binds,
    })
}

fn filter_equip_level(options: &[String]) -> Option<FilterBindingInfo> {
    filter_generic_range("i.equip_level", options)
}

fn filter_ilevel(options: &[String]) -> Option<FilterBindingInfo> {
    filter_generic_range("i.item_level", options)
}

fn filter_ui_category(options: &[String]) -> Option<FilterBindingInfo> {
    let Some(FilterBindingInfo { clause, binds }) = filter_generic_regex("c", options) else {
        return None;
    };

    Some(FilterBindingInfo {
        clause: format!(
            "i.ui_category IN (
                SELECT c.id
                FROM {} AS c
                WHERE {}
            )",
            UiCategoryTable::SQL_TABLE_NAME,
            clause
        ),
        binds,
    })
}

fn filter_contains(options: &[String]) -> Option<FilterBindingInfo> {
    let Some(FilterBindingInfo { clause, binds }) = filter_generic_regex("i_g", options) else {
        return None;
    };

    Some(FilterBindingInfo {
        clause: format!(
            "i.id IN (
                SELECT g.item_id
                FROM {} AS g
                INNER JOIN {} AS i_g ON g.input_id = i_g.id
                WHERE {}
            )",
            IngredientTable::SQL_TABLE_NAME,
            ItemInfoTable::SQL_TABLE_NAME,
            clause
        ),
        binds,
    })
}

fn filter_includes(options: &[String]) -> Option<FilterBindingInfo> {
    let Some(FilterBindingInfo { clause, binds }) = filter_generic_regex("i_n", options) else {
        return None;
    };

    Some(FilterBindingInfo {
        clause: format!(
            "i.id IN (
                SELECT n.item_id
                FROM {} AS n
                INNER JOIN {} AS i_n ON n.input_id = i_n.id
                WHERE {}
            )",
            InputIdsTable::SQL_TABLE_NAME,
            ItemInfoTable::SQL_TABLE_NAME,
            clause
        ),
        binds,
    })
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
fn filter_noop(_options: &[String]) -> Option<FilterBindingInfo> {
    None
}

////////////////////////////////////////////////////////////
