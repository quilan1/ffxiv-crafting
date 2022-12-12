import { ItemInfo, Listing } from "./custom_info.js";

const HOMEWORLD = "Seraph";

export type Quality<T> = {
    aq: T,
    nq: T,
    hq?: T,
}

export default class Statistics {
    readonly minBuyPrice?: Quality<number>;
    readonly homeworldMedSellPrice?: Quality<number>;
    readonly homeworldVelocityDay?: Quality<number>;
    readonly homeworldVelocityWeek?: Quality<number>;
    readonly homeworldVelocityWeeks?: Quality<number>;

    constructor(itemInfo: ItemInfo) {
        const isHomeworld = (item: Listing) => item.world === HOMEWORLD;
        const toIdent = (item: Listing) => item;
        const toPrice = (item: Listing) => item.price;
        const median = (items: number[]) => { items.sort((a,b) => a - b); return items[(items.length / 3) | 0]; };
        const min = (items: number[]) => items.reduce((prev: number | undefined, cur: number) => (prev === undefined || cur < prev) ? cur : prev, undefined) as number;
        const velocity = (days: number) => (listings: Listing[]) => calculateVelocity(listings, days);

        this.minBuyPrice = generateQuality(itemInfo.name, itemInfo.listings, [], toPrice, min);
        this.homeworldMedSellPrice = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld], toPrice, median);
        this.homeworldVelocityDay = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld], toIdent, velocity(1.0));
        this.homeworldVelocityWeek = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld], toIdent, velocity(7.0));
        this.homeworldVelocityWeeks = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld], toIdent, velocity(14.0));
    }
}

function generateQuality<T, O>(name: string, listings: Listing[], filter_fns: ((item: Listing) => boolean)[], map_fn: (item: Listing) => T, reduce_fn: (items: T[]) => O): Quality<O> | undefined {
    if (listings.length === 0) {
        return undefined;
    }

    const isHq = (item: Listing) => item.is_hq;
    const isNq = (item: Listing) => !item.is_hq;

    return {
        aq: reduceListings(`${name} aq`, listings, filter_fns, map_fn, reduce_fn) as O,
        nq: reduceListings(`${name} nq`, listings, [...filter_fns, isNq], map_fn, reduce_fn) as O,
        hq: reduceListings(`${name} hq`, listings, [...filter_fns, isHq], map_fn, reduce_fn),
    };
}

function reduceListings<T, O>(name: string, listings: Listing[], filter_fns: ((item: Listing) => boolean)[], map_fn: (item: Listing) => T, reduce_fn: (items: T[]) => O): O | undefined {
    const filteredListings = [];

    for (const listing of listings) {
        if (!filter_fns.every(filter => filter(listing))) {
            continue;
        }

        filteredListings.push(map_fn(listing));
    }

    return (filteredListings.length > 0) ? reduce_fn(filteredListings) : undefined;
}

function calculateVelocity(allListings: Listing[], days: number): number {
    const postingToDays = (posting: number) => (Date.now() / 1000.0 - posting) / 3600.0 / 24.0;
    let listings = allListings.filter(listing => postingToDays(listing.posting) <= days);

    const totalCount = listings.reduce((prev, item) => prev + item.count, 0);
    const minPosting = Math.min(...listings.map(item => item.posting));
    return totalCount / postingToDays(minPosting);
}
