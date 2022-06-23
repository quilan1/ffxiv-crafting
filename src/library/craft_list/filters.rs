use regex::Regex;

use crate::library::{ItemInfo, Library};

type FilterOptions = Vec<String>;

#[derive(Clone)]
pub struct Filter {
    pub ftype: String,
    pub options: FilterOptions,
}

impl Filter {
    pub fn apply_filters<'a>(
        library: &Library,
        mut items: Vec<&'a ItemInfo>,
        filters: &str,
    ) -> (Vec<&'a ItemInfo>, Vec<Filter>) {
        let filters = Self::filters(filters);
        let mut result_filters = Vec::new();
        for Filter { ftype, options } in filters {
            items = match &ftype[..] {
                ":name" => Self::filter_name(library, options, items),
                ":rlevel" => Self::filter_recipe_level(library, options, items),
                ":elevel" => Self::filter_equip_level(library, options, items),
                ":ilevel" => Self::filter_ilevel(library, options, items),
                ":cat" => Self::filter_ui_category(library, options, items),
                ":is_leve" => Self::filter_leve(library, options, items),
                ":contains" => Self::filter_contains(library, options, items),
                ":count" => {
                    result_filters.push(Filter { ftype, options });
                    continue;
                }
                f @ _ => {
                    println!("Unknown filter: {}", f);
                    items
                }
            }
        }

        (items, result_filters)
    }

    fn filter_name<'a>(
        _: &Library,
        options: FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let re = Regex::new(&options.join("|")).unwrap();

        items
            .into_iter()
            .filter(|item| re.is_match(&item.name))
            .collect::<Vec<_>>()
    }

    fn filter_recipe_level<'a>(
        library: &Library,
        options: FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let levels = options
            .into_iter()
            .map(|level| level.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let min_level = levels[0];
        let max_level = *levels.last().unwrap();

        items
            .into_iter()
            .filter(|item| {
                let recipe = &library.all_recipes[&item.id];
                let recipe_level = &library.all_recipe_levels[&recipe.level_id];
                recipe_level.level >= min_level && recipe_level.level <= max_level
            })
            .collect::<Vec<_>>()
    }

    fn filter_equip_level<'a>(
        _: &Library,
        options: FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let levels = options
            .into_iter()
            .map(|level| level.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let min_level = levels[0];
        let max_level = *levels.last().unwrap();

        items
            .into_iter()
            .filter(|item| item.equip_level >= min_level && item.equip_level <= max_level)
            .collect::<Vec<_>>()
    }

    fn filter_ilevel<'a>(
        _: &Library,
        options: FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let levels = options
            .into_iter()
            .map(|level| level.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let min_level = levels[0];
        let max_level = *levels.last().unwrap();

        items
            .into_iter()
            .filter(|item| item.ilevel >= min_level && item.ilevel <= max_level)
            .collect::<Vec<_>>()
    }

    fn filter_ui_category<'a>(
        library: &Library,
        options: FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let categories = options;

        items
            .into_iter()
            .filter(|item| categories.contains(&library.all_ui_categories[&item.ui_category]))
            .collect::<Vec<_>>()
    }

    fn filter_leve<'a>(
        library: &Library,
        options: FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let categories = options
            .iter()
            .map(|cat| cat.as_str())
            .collect::<Vec<_>>();
        let all_leve_items = library.all_leves.all_item_ids();

        items
            .into_iter()
            .filter(|item| all_leve_items.contains(&item.id))
            .filter(|item| {
                let leve_ids = library.all_leves.get_by_item_id(&item.id).unwrap();
                leve_ids
                    .iter()
                    .map(|leve_id| &library.all_leves[&leve_id].jobs)
                    .any(|jobs| library.all_job_categories[&jobs].matches_any(&categories))
            })
            .collect::<Vec<_>>()
    }

    fn filter_contains<'a>(
        library: &Library,
        options: FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let re = Regex::new(&options.join("|")).unwrap();

        items
            .into_iter()
            .filter(|item| {
                match library.all_recipes.get(&item.id) {
                    None => false,
                    Some(recipe) => {
                        recipe.inputs.iter().any(|input| {
                            match library.all_items.items.get(&input.item_id) {
                                None => false,
                                Some(input_item) => re.is_match(&input_item.name)
                            }
                        })
                    }
                }
            }).collect::<Vec<_>>()
    }

    fn filters(filters: &str) -> Vec<Filter> {
        filters
            .split(",")
            .map(|filter| {
                let filter = filter.trim();
                let contents = filter.split(" ").collect::<Vec<_>>();
                let (ftype, options) = if contents.len() > 1 {
                    (
                        contents[0].to_string(),
                        contents[1..]
                            .join(" ")
                            .split("|")
                            .map(|filter| filter.trim())
                            .filter(|filter| filter.len() > 0)
                            .map(|filter| filter.to_string())
                            .collect::<Vec<_>>(),
                    )
                } else {
                    (contents[0].to_string(), Vec::new())
                };
                Filter { ftype, options }
            })
            .collect::<Vec<_>>()
    }
}
