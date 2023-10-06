export type Id = number;
export type IdChain = number[];

export type RecipeData = {
    item_id: Id,
    count: number,
}

export type Recipe = {
    inputs: RecipeData[],
    outputs: number,
}

export type Listing = {
    price: number,
    count: number,
    is_hq: boolean,
    world?: string,
    name?: string,
    posting: number,
}

export type ItemInfo = {
    item_id: number,
    name: string,
    listings: Listing[],
    history: Listing[],
    recipe?: Recipe,
}
