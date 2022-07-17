use anyhow::Result;
use std::{collections::BTreeSet, fs::read_to_string, path::Path};

use crate::util::{item, library};

use super::Filter;

#[derive(Default)]
pub struct CraftList {
    pub craft_groups: Vec<CraftGroup>,
}

pub struct CraftInfo {
    pub item_id: u32,
    pub filters: Vec<Filter>,
}

#[derive(Default)]
pub struct CraftGroup {
    pub heading: String,
    pub crafts: Vec<CraftInfo>,
    pub filters: Vec<Filter>,
}

impl CraftList {
    pub fn from_path<P: AsRef<Path>>(path: P, is_all_items: bool) -> Result<Self> {
        let contents = read_to_string(path.as_ref())?;
        let lines = contents
            .split("\r\n")
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();

        let item_list = if is_all_items {
            library().all_items.items.values().collect::<Vec<_>>()
        } else {
            library().all_craftable_items()
        };

        let mut craft_groups = Vec::new();
        let mut cur_group: Option<CraftGroup> = None;
        let mut skip = false;

        for line in lines {
            if &line[..1] == "=" {
                if let Some(group) = cur_group {
                    craft_groups.push(group);
                }
                cur_group = Some(CraftGroup {
                    heading: line[2..].into(),
                    ..Default::default()
                });
                skip = false;
                continue;
            }

            if skip || &line[..1] == "#" {
                continue;
            }

            let group = cur_group.get_or_insert(Default::default());
            match &line[..1] {
                "S" => skip = true,
                "X" => break,
                ">" => {
                    let filters = Filter::new(&line[2..]);
                    group.filters.extend(filters);
                }
                _ => {
                    let (items, filters) = Filter::apply_filters(item_list.clone(), &line);
                    for item in items {
                        // println!("Adding filter match: {}", item.name);
                        group.crafts.push(CraftInfo {
                            item_id: item.id,
                            filters: filters.clone(),
                        });
                    }
                }
            }
        }

        if let Some(group) = cur_group {
            craft_groups.push(group);
        }

        Ok(Self { craft_groups })
    }

    pub fn all_craft_item_ids(&self) -> Vec<u32> {
        fn push_ids(ids: &mut Vec<u32>, item_id: u32) {
            ids.push(item_id);
            if !library().all_recipes.contains_item_id(item_id) {
                return;
            }

            for input in &library().all_recipes[&item_id].inputs {
                push_ids(ids, input.item_id);
            }
        }

        self.craft_groups
            .iter()
            .map(|craft_group| &craft_group.crafts)
            .flatten()
            .map(|craft| {
                let mut item_ids = Vec::new();
                push_ids(&mut item_ids, craft.item_id);
                item_ids
            })
            .flatten()
            .filter(|item_id| !item(item_id).is_untradable)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    }

    pub fn perform_filtering() {}
}
