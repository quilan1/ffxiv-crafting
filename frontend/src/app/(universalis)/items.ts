export type Id = number;
export type IdChain = number[];

export interface Ingredient {
    itemId: Id,
    count: number,
}

export interface Recipe {
    inputs: Ingredient[],
    outputs: number,
    level: number,
}

export interface Listing {
    price: number,
    count: number,
    isHq: boolean,
    world?: string,
    name?: string,
    daysSince: number,
}

export interface ItemInfo {
    itemId: number,
    name: string,
    listings: Listing[],
    history: Listing[],
    recipe?: Recipe,
}
