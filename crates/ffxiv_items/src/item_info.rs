use crate::Recipe;

#[derive(Clone)]
pub struct ItemInfo {
    pub id: u32,
    pub name: String,
    pub recipe: Option<Recipe>,
}
