use std::collections::HashMap;

use log::info;
use regex::Regex;

use crate::{ItemInfo, Library};

type FilterOptions = Vec<String>;

#[derive(Clone)]
pub struct Filter {
    pub tag: String,
    pub options: FilterOptions,
}

type FilterFn =
    for<'a, 'b, 'c> fn(library: &'c Library, &'a [String], &'b mut Vec<&'c ItemInfo>) -> ();

impl Filter {
    pub fn apply_filter_str<'a>(
        library: &'a Library,
        filter_str: &str,
        mut items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        if filter_str.trim().is_empty() {
            return Vec::new();
        }

        let filters = Self::from_str(filter_str);
        let filter_functions = Self::filter_functions();
        for Filter { tag, options } in filters {
            match filter_functions.get(&tag[..]) {
                Some(func) => func(library, &options, &mut items),
                None => {
                    if tag.chars().nth(0).unwrap_or(' ') == ':' {
                        info!(target: "ffxiv_items", "Invalid filter tag: {tag}");
                        continue;
                    }
                    info!(target: "ffxiv_items", "Missing filter tag: {tag}, interpreting it as a :name filter",);
                    let mut new_options = options.clone();
                    if let Some(option) = new_options.first_mut() {
                        *option = format!("{tag} {option}");
                    } else {
                        new_options.push(tag);
                    }
                    filter_name(library, &new_options, &mut items)
                }
            }
        }
        items
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
        map.insert(":leve", filter_leve);

        // Normal filters
        map.insert(":name", filter_name);
        map.insert(":rlevel", filter_recipe_level);
        map.insert(":elevel", filter_equip_level);
        map.insert(":ilevel", filter_ilevel);
        map.insert(":cat", filter_ui_category);
        map.insert(":contains", filter_contains);

        // Result filters
        map.insert(":count", filter_noop);
        map.insert(":limit", filter_noop);
        map.insert(":min_velocity", filter_noop);

        map
    }
}

////////////////////////////////////////////////////////////

fn filter_name<'a>(_library: &'a Library, options: &[String], items: &mut Vec<&'a ItemInfo>) {
    if options.is_empty() {
        return;
    }

    let re = options.join("|").replace(' ', "\\s");
    let re = Regex::new(&re).unwrap();

    items.retain(|item| re.is_match(&item.name));
}

fn filter_recipe_level<'a>(
    _library: &'a Library,
    options: &[String],
    items: &mut Vec<&'a ItemInfo>,
) {
    let levels = options
        .iter()
        .filter_map(|level| level.parse::<u32>().ok())
        .collect::<Vec<_>>();
    let min_level = levels.first().cloned().unwrap_or(u32::MIN);
    let max_level = levels.last().cloned().unwrap_or(u32::MAX);

    items.retain(|item| {
        item.recipe.as_ref().map_or(false, |recipe| {
            recipe.level >= min_level && recipe.level <= max_level
        })
    });
}

fn filter_equip_level<'a>(
    _library: &'a Library,
    options: &[String],
    items: &mut Vec<&'a ItemInfo>,
) {
    if options.is_empty() {
        return;
    }

    let levels = options
        .iter()
        .filter_map(|level| level.parse::<u32>().ok())
        .collect::<Vec<_>>();
    let min_level = levels.first().cloned().unwrap_or(u32::MIN);
    let max_level = levels.last().cloned().unwrap_or(u32::MAX);

    items.retain(|item| item.equip_level >= min_level && item.equip_level <= max_level);
}

fn filter_ilevel<'a>(_library: &'a Library, options: &[String], items: &mut Vec<&'a ItemInfo>) {
    if options.is_empty() {
        return;
    }

    let levels = options
        .iter()
        .filter_map(|level| level.parse::<u32>().ok())
        .collect::<Vec<_>>();
    let min_level = levels.first().cloned().unwrap_or(u32::MIN);
    let max_level = levels.last().cloned().unwrap_or(u32::MAX);

    items.retain(|item| item.ilevel >= min_level && item.ilevel <= max_level);
}

fn filter_ui_category<'a>(library: &'a Library, options: &[String], items: &mut Vec<&'a ItemInfo>) {
    if options.is_empty() {
        return;
    }

    let categories = options.iter().map(|cat| cat.as_str()).collect::<Vec<_>>();
    items.retain(|item| categories.contains(&library.ui_category_unchecked(item.ui_category)));
}

fn filter_leve<'a>(library: &'a Library, options: &[String], items: &mut Vec<&'a ItemInfo>) {
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

fn filter_contains<'a>(library: &'a Library, options: &[String], items: &mut Vec<&'a ItemInfo>) {
    if options.is_empty() {
        return;
    }

    let re = Regex::new(&options.join("|")).unwrap();
    items.retain(|&item| {
        item.all_recipe_input_ids(library, item)
            .iter()
            .any(|id| re.is_match(&library.item_info(id).name))
    });
}

#[allow(clippy::ptr_arg)]
fn filter_noop<'a>(_library: &'a Library, _options: &[String], _items: &mut Vec<&'a ItemInfo>) {}

////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::Library;

    use super::*;

    fn item_names<'a>(library: &'a Library, items: Vec<&ItemInfo>) -> Vec<&'a str> {
        let item_name = |id| library.item_name(id);
        items.into_iter().map(item_name).collect()
    }

    #[test]
    fn test_empty_filter_string() {
        let library = Library::initialize_test_data();

        let items = Filter::apply_filter_str(&library, "", library.all_items());
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_missing_tag() {
        let library = Library::initialize_test_data();

        let items = Filter::apply_filter_str(&library, "Test 1", library.all_items());
        assert_eq!(items.len(), 1);
        assert_eq!(item_names(&library, items), vec!["Test 1"]);

        let items = Filter::apply_filter_str(&library, "Test", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Test 1", "Test 2"]);
    }

    #[test]
    fn test_name() {
        let library = Library::initialize_test_data();

        let items = Filter::apply_filter_str(&library, ":name Test", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Test 1", "Test 2"]);

        let items = Filter::apply_filter_str(&library, ":name Extra", library.all_items());
        assert_eq!(items.len(), 1);
        assert_eq!(item_names(&library, items), vec!["Extra"]);

        let items = Filter::apply_filter_str(&library, ":name", library.all_items());
        assert_eq!(items.len(), 6);
    }

    #[test]
    fn test_ui_categories() {
        let library = Library::initialize_test_data();

        let items = Filter::apply_filter_str(&library, ":cat cat 1", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Test 1", "Test 2"]);

        let items = Filter::apply_filter_str(&library, ":cat cat 2", library.all_items());
        assert_eq!(items.len(), 1);
        assert_eq!(item_names(&library, items), vec!["Extra"]);

        let items = Filter::apply_filter_str(&library, ":cat XXXX", library.all_items());
        assert_eq!(items.len(), 0);

        let items = Filter::apply_filter_str(&library, ":cat", library.all_items());
        assert_eq!(items.len(), 6);
    }

    #[test]
    fn test_contains() {
        let library = Library::initialize_test_data();

        let items = Filter::apply_filter_str(&library, ":contains Base 1", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Base 3", "Test 1"]);

        let items = Filter::apply_filter_str(&library, ":contains Extra", library.all_items());
        assert_eq!(items.len(), 0);

        let items = Filter::apply_filter_str(&library, ":contains", library.all_items());
        assert_eq!(items.len(), 6);
    }

    #[test]
    fn test_recipe_level() {
        let library = Library::initialize_test_data();

        let items = Filter::apply_filter_str(&library, ":rlevel 81|81", library.all_items());
        assert_eq!(items.len(), 1);
        assert_eq!(item_names(&library, items), vec!["Base 3"]);

        let items = Filter::apply_filter_str(&library, ":rlevel 81", library.all_items());
        assert_eq!(items.len(), 1);
        assert_eq!(item_names(&library, items), vec!["Base 3"]);

        let items = Filter::apply_filter_str(&library, ":rlevel 84|84", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Test 1", "Test 2"]);

        let items = Filter::apply_filter_str(&library, ":rlevel 84", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Test 1", "Test 2"]);

        let items = Filter::apply_filter_str(&library, ":rlevel 1|99", library.all_items());
        assert_eq!(items.len(), 4);
        assert_eq!(
            item_names(&library, items),
            vec!["Base 3", "Test 1", "Test 2", "Extra"]
        );

        let items = Filter::apply_filter_str(&library, ":rlevel XXXX", library.all_items());
        assert_eq!(items.len(), 4);
        assert_eq!(
            item_names(&library, items),
            vec!["Base 3", "Test 1", "Test 2", "Extra"]
        );

        let items = Filter::apply_filter_str(&library, ":rlevel", library.all_items());
        assert_eq!(items.len(), 4);
        assert_eq!(
            item_names(&library, items),
            vec!["Base 3", "Test 1", "Test 2", "Extra"]
        );
    }

    #[test]
    fn test_item_ilevel() {
        let library = Library::initialize_test_data();

        let items = Filter::apply_filter_str(&library, ":ilevel 500|599", library.all_items());
        assert_eq!(items.len(), 1);
        assert_eq!(item_names(&library, items), vec!["Extra"]);

        let items = Filter::apply_filter_str(&library, ":ilevel 530", library.all_items());
        assert_eq!(items.len(), 1);
        assert_eq!(item_names(&library, items), vec!["Extra"]);

        let items = Filter::apply_filter_str(&library, ":ilevel 600|699", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Test 1", "Test 2"]);

        let items = Filter::apply_filter_str(&library, ":ilevel 660", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Test 1", "Test 2"]);

        let items = Filter::apply_filter_str(&library, ":ilevel XXXX", library.all_items());
        assert_eq!(items.len(), 6);

        let items = Filter::apply_filter_str(&library, ":ilevel", library.all_items());
        assert_eq!(items.len(), 6);
    }

    #[test]
    fn test_item_level() {
        let library = Library::initialize_test_data();

        let items = Filter::apply_filter_str(&library, ":elevel 90|90", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Test 1", "Test 2"]);

        let items = Filter::apply_filter_str(&library, ":elevel 90", library.all_items());
        assert_eq!(items.len(), 2);
        assert_eq!(item_names(&library, items), vec!["Test 1", "Test 2"]);

        let items = Filter::apply_filter_str(&library, ":elevel XXXX", library.all_items());
        assert_eq!(items.len(), 6);

        let items = Filter::apply_filter_str(&library, ":elevel", library.all_items());
        assert_eq!(items.len(), 6);
    }

    // TODO: Make this later
    // #[test]
    // fn test_item_leve() {}
}
