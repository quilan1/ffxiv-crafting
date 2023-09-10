use regex::Regex;

use crate::{library, parsers::UiCategoryList, ItemInfo, Recipe};

type FilterOptions = Vec<String>;

#[derive(Clone)]
pub struct Filter {
    pub ftype: String,
    pub options: FilterOptions,
}

impl Filter {
    pub fn new(filters: &str) -> Vec<Filter> {
        filters
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
                Filter { ftype, options }
            })
            .collect::<Vec<_>>()
    }

    pub fn apply_filters(
        mut items: Vec<&'static ItemInfo>,
        filters: &str,
    ) -> (Vec<&'static ItemInfo>, Vec<Filter>) {
        let filters = Self::new(filters);
        let mut result_filters = Vec::new();
        for Filter { ftype, options } in filters {
            match &ftype[..] {
                ":name" => Self::filter_name(&options, &mut items),
                ":rlevel" => Self::filter_recipe_level(&options, &mut items),
                ":elevel" => Self::filter_equip_level(&options, &mut items),
                ":ilevel" => Self::filter_ilevel(&options, &mut items),
                ":cat" => Self::filter_ui_category(&options, &mut items),
                ":is_leve" => Self::filter_leve(&options, &mut items),
                ":contains" => Self::filter_contains(&options, &mut items),
                ":count" | ":limit" | ":min_velocity" => {
                    result_filters.push(Filter { ftype, options });
                    continue;
                }
                f => println!("Unknown filter: {f}"),
            }
        }

        (items, result_filters)
    }

    fn filter_name(options: &FilterOptions, items: &mut Vec<&'static ItemInfo>) {
        let re = options.join("|").replace(' ', "\\s");
        let re = Regex::new(&re).unwrap();

        items.retain(|item| re.is_match(&item.name));
    }

    fn filter_recipe_level(options: &FilterOptions, items: &mut Vec<&'static ItemInfo>) {
        let levels = options
            .iter()
            .map(|level| level.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let min_level = levels[0];
        let max_level = *levels.last().unwrap();

        items.retain(|item| {
            item.recipe
                .as_ref()
                .map_or(false, |Recipe { level_info, .. }| {
                    level_info.as_ref().map_or(false, |info| {
                        info.level >= min_level && info.level <= max_level
                    })
                })
        });
    }

    fn filter_equip_level(options: &FilterOptions, items: &mut Vec<&'static ItemInfo>) {
        let levels = options
            .iter()
            .map(|level| level.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let min_level = levels[0];
        let max_level = *levels.last().unwrap();

        items.retain(|item| item.equip_level >= min_level && item.equip_level <= max_level);
    }

    fn filter_ilevel(options: &FilterOptions, items: &mut Vec<&'static ItemInfo>) {
        let levels = options
            .iter()
            .map(|level| level.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let min_level = levels[0];
        let max_level = *levels.last().unwrap();

        items.retain(|item| item.ilevel >= min_level && item.ilevel <= max_level);
    }

    fn filter_ui_category(options: &FilterOptions, items: &mut Vec<&'static ItemInfo>) {
        let categories = options.iter().map(|cat| cat.as_str()).collect::<Vec<_>>();
        items.retain(|item| categories.contains(&UiCategoryList::get_unchecked(item.ui_category)));
    }

    fn filter_leve(options: &FilterOptions, items: &mut Vec<&'static ItemInfo>) {
        let categories = options;
        let all_leve_items = library().all_leves.all_item_ids();

        items.retain(|item| all_leve_items.contains(&item.id));
        items.retain(|item| {
            let leve_ids = library().all_leves.get_by_item_id(item.id).unwrap();
            leve_ids
                .iter()
                .map(|leve_id| &library().all_leves[leve_id].jobs)
                .any(|jobs| library().all_job_categories[jobs].matches_any(categories))
        });
    }

    fn filter_contains(options: &FilterOptions, items: &mut Vec<&'static ItemInfo>) {
        let re = Regex::new(&options.join("|")).unwrap();

        items.retain(|item| {
            item.recipe.as_ref().map_or(false, |recipe| {
                recipe.inputs.iter().any(|input| {
                    ItemInfo::get_checked(input)
                        .map_or(false, |input_item| re.is_match(&input_item.name))
                })
            })
        });
    }
}
