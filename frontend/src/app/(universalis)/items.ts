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

export interface BaseItemInfo {
    itemId: number,
    name: string,
    recipe?: Recipe,
}

export type ItemInfo = BaseItemInfo & {
    listings: Listing[],
    history: Listing[],
}