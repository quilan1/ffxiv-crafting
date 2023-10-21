use crate::Recipe;

/// The base information for an item.
#[derive(Clone)]
pub struct ItemInfo {
    /// The unique numeric value for the item.
    pub id: u32,
    /// The displayed name for the item.
    pub name: String,
    /// The recipe for the item, if it is craftable.
    pub recipe: Option<Recipe>,
}
