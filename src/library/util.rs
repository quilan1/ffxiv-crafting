use super::{Ingredient, MarketBoardAnalysis, RecursiveMarketBoardAnalysis};

pub trait ItemId {
    fn item_id(&self) -> u32;
}

impl ItemId for u32 {
    fn item_id(&self) -> u32 {
        *self
    }
}

impl ItemId for &u32 {
    fn item_id(&self) -> u32 {
        **self
    }
}

impl ItemId for MarketBoardAnalysis {
    fn item_id(&self) -> u32 {
        self.item_id
    }
}

impl ItemId for &MarketBoardAnalysis {
    fn item_id(&self) -> u32 {
        self.item_id
    }
}

impl ItemId for RecursiveMarketBoardAnalysis {
    fn item_id(&self) -> u32 {
        self.analysis.item_id
    }
}

impl ItemId for &RecursiveMarketBoardAnalysis {
    fn item_id(&self) -> u32 {
        self.analysis.item_id
    }
}

impl ItemId for Ingredient {
    fn item_id(&self) -> u32 {
        self.item_id
    }
}

impl ItemId for &Ingredient {
    fn item_id(&self) -> u32 {
        self.item_id
    }
}
