use anyhow::Result;
use regex::Regex;

use crate::library::ItemInfo;
use crate::util::{item_checked, library};

type FilterOptions = Vec<String>;

#[derive(PartialEq)]
pub enum QualityFilter {
    Any,
    NQ,
    HQ,
}

impl Default for QualityFilter {
    fn default() -> Self {
        Self::Any
    }
}

#[derive(Default)]
pub struct AnalysisFilters {
    pub quality: QualityFilter,
    pub count: Option<u32>,
    pub limit: Option<u32>,
    pub min_velocity: Option<f32>,
    pub always_top: bool,
}

impl AnalysisFilters {
    #[allow(dead_code)]
    pub fn new(group_filters: &[Filter], item_filters: &[Filter]) -> Result<Self> {
        let mut all_filters = group_filters.to_owned();
        all_filters.extend(item_filters.to_owned());
        Self::from_filters(&all_filters)
    }

    #[allow(dead_code)]
    pub fn from_filters(all_filters: &Vec<Filter>) -> Result<Self> {
        let mut filters = AnalysisFilters::default();
        for Filter { ftype, options } in all_filters {
            match &ftype[..] {
                ":count" => filters.count = Some(options.join("").parse::<u32>()?),
                ":limit" => filters.limit = Some(options.join("").parse::<u32>()?),
                ":min_velocity" => filters.min_velocity = Some(options.join("").parse::<f32>()?),
                ":only_hq" => filters.quality = QualityFilter::HQ,
                ":as_nq" => filters.quality = QualityFilter::NQ,
                ":always_top" => filters.always_top = true,
                _ => {}
            }
        }

        Ok(filters)
    }
}

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

    pub fn apply_filters<'a>(
        mut items: Vec<&'a ItemInfo>,
        filters: &str,
    ) -> (Vec<&'a ItemInfo>, Vec<Filter>) {
        let filters = Self::new(filters);
        let mut result_filters = Vec::new();
        for Filter { ftype, options } in filters {
            items = match &ftype[..] {
                ":name" => Self::filter_name(&options, items),
                ":rlevel" => Self::filter_recipe_level(&options, items),
                ":elevel" => Self::filter_equip_level(&options, items),
                ":ilevel" => Self::filter_ilevel(&options, items),
                ":cat" => Self::filter_ui_category(&options, items),
                ":is_leve" => Self::filter_leve(&options, items),
                ":contains" => Self::filter_contains(&options, items),
                ":count" | ":limit" | ":min_velocity" => {
                    result_filters.push(Filter { ftype, options });
                    continue;
                }
                f => {
                    println!("Unknown filter: {f}");
                    items
                }
            }
        }

        (items, result_filters)
    }

    fn filter_name<'a>(options: &FilterOptions, items: Vec<&'a ItemInfo>) -> Vec<&'a ItemInfo> {
        let re = options.join("|").replace(' ', "\\s");
        let re = Regex::new(&re).unwrap();

        items
            .into_iter()
            .filter(|item| re.is_match(&item.name))
            .collect::<Vec<_>>()
    }

    fn filter_recipe_level<'a>(
        options: &FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let levels = options
            .iter()
            .map(|level| level.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let min_level = levels[0];
        let max_level = *levels.last().unwrap();

        items
            .into_iter()
            .filter(|item| {
                if let Some(recipe) = library().all_recipes.get(item.id) {
                    let recipe_level = &library().all_recipe_levels[&recipe.level_id];
                    recipe_level.level >= min_level && recipe_level.level <= max_level
                } else {
                    false
                }
            })
            .collect::<Vec<_>>()
    }

    fn filter_equip_level<'a>(
        options: &FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let levels = options
            .iter()
            .map(|level| level.parse::<u32>().unwrap())
            .collect::<Vec<_>>();
        let min_level = levels[0];
        let max_level = *levels.last().unwrap();

        items
            .into_iter()
            .filter(|item| item.equip_level >= min_level && item.equip_level <= max_level)
            .collect::<Vec<_>>()
    }

    fn filter_ilevel<'a>(options: &FilterOptions, items: Vec<&'a ItemInfo>) -> Vec<&'a ItemInfo> {
        let levels = options
            .iter()
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
        options: &FilterOptions,
        items: Vec<&'a ItemInfo>,
    ) -> Vec<&'a ItemInfo> {
        let categories = options;

        items
            .into_iter()
            .filter(|item| categories.contains(&library().all_ui_categories[&item.ui_category]))
            .collect::<Vec<_>>()
    }

    fn filter_leve<'a>(options: &FilterOptions, items: Vec<&'a ItemInfo>) -> Vec<&'a ItemInfo> {
        let categories = options.iter().map(String::as_str).collect::<Vec<_>>();
        let all_leve_items = library().all_leves.all_item_ids();

        items
            .into_iter()
            .filter(|item| all_leve_items.contains(&item.id))
            .filter(|item| {
                let leve_ids = library().all_leves.get_by_item_id(item.id).unwrap();
                leve_ids
                    .iter()
                    .map(|leve_id| &library().all_leves[leve_id].jobs)
                    .any(|jobs| library().all_job_categories[jobs].matches_any(&categories))
            })
            .collect::<Vec<_>>()
    }

    fn filter_contains<'a>(options: &FilterOptions, items: Vec<&'a ItemInfo>) -> Vec<&'a ItemInfo> {
        let re = Regex::new(&options.join("|")).unwrap();

        items
            .into_iter()
            .filter(|item| {
                library().all_recipes.get(item.id).map_or(false, |recipe| {
                    recipe.inputs.iter().any(|input| {
                        item_checked(input)
                            .map_or(false, |input_item| re.is_match(&input_item.name))
                    })
                })
            })
            .collect::<Vec<_>>()
    }
}
