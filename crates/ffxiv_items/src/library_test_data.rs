use crate::{
    parsers::{ItemList, UiCategoryList},
    recipe::{Ingredient, Recipe},
    ItemInfo, Library,
};

impl Library {
    pub(crate) fn initialize_test_data() -> Library {
        let categories = UiCategoryList::from(&["base 1", "base 2", "cat 1", "cat 2"][..]);
        let items = vec![
            ItemInfo {
                id: 1,
                name: "Base 1".into(),
                ui_category: categories["base 1"],
                ..Default::default()
            },
            ItemInfo {
                id: 2,
                name: "Base 2".into(),
                ui_category: categories["base 1"],
                ..Default::default()
            },
            ItemInfo {
                id: 3,
                name: "Base 3".into(),
                ui_category: categories["base 2"],
                recipe: Some(Recipe {
                    output: Ingredient {
                        count: 2,
                        item_id: 3,
                    },
                    inputs: vec![
                        Ingredient {
                            count: 1,
                            item_id: 1,
                        },
                        Ingredient {
                            count: 1,
                            item_id: 2,
                        },
                    ],
                    level: 81,
                    stars: 4,
                }),
                ..Default::default()
            },
            ItemInfo {
                id: 4,
                name: "Test 1".into(),
                ui_category: categories["cat 1"],
                ilevel: 660,
                equip_level: 90,
                is_untradable: false,
                recipe: Some(Recipe {
                    output: Ingredient {
                        count: 1,
                        item_id: 4,
                    },
                    inputs: vec![Ingredient {
                        count: 2,
                        item_id: 3,
                    }],
                    level: 84,
                    stars: 5,
                }),
            },
            ItemInfo {
                id: 5,
                name: "Test 2".into(),
                ui_category: categories["cat 1"],
                ilevel: 660,
                equip_level: 90,
                is_untradable: false,
                recipe: Some(Recipe {
                    output: Ingredient {
                        count: 1,
                        item_id: 5,
                    },
                    inputs: vec![Ingredient {
                        count: 1,
                        item_id: 2,
                    }],
                    level: 84,
                    stars: 5,
                }),
            },
            ItemInfo {
                id: 6,
                name: "Extra".into(),
                ui_category: categories["cat 2"],
                ilevel: 530,
                equip_level: 80,
                is_untradable: false,
                recipe: Some(Recipe {
                    output: Ingredient {
                        count: 1,
                        item_id: 6,
                    },
                    inputs: vec![Ingredient {
                        count: 4,
                        item_id: 2,
                    }],
                    level: 85,
                    stars: 5,
                }),
            },
        ];

        Library {
            all_items: ItemList::from(items),
            all_ui_categories: categories,
            // TODO: Other info
            ..Default::default()
        }
    }
}
