use crate::Recipe;

#[derive(Clone, Default)]
pub struct ItemDBInfo {
    pub id: u32,
    pub name: String,
    pub ui_category: u32,
    pub ilevel: u32,
    pub equip_level: u32,
    pub recipe: Option<Recipe>,
}

#[derive(Clone, Default)]
pub struct ItemInfo {
    pub id: u32,
    pub name: String,
    pub recipe: Option<Recipe>,
}
