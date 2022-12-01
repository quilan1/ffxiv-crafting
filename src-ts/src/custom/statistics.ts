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
    readonly homeworldVelocity?: Quality<number>;

    constructor(itemInfo: ItemInfo) {
        const isHomeworld = (item: Listing) => item.world === HOMEWORLD;
        const toIdent = (item: Listing) => item;
        const toPrice = (item: Listing) => item.price;
        const median = (items: number[]) => { items.sort(); return items[(items.length / 2) | 0]; };
        const min = (items: number[]) => items.reduce((prev: number | undefined, cur: number) => (prev === undefined || cur < prev) ? cur : prev, undefined) as number;
    
        this.minBuyPrice = generateQuality(itemInfo.name, itemInfo.listings, [], toPrice, min);
        this.homeworldMedSellPrice = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld], toPrice, median);
        this.homeworldVelocity = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld], toIdent, calculateVelocity);
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

function calculateVelocity(items: Listing[]): number {
    const totalCount = items.reduce((prev, item) => prev + item.count, 0);

    const minPosting = Math.min(...items.map(item => item.posting));
    const timeDiffMillis = Date.now() / 1000.0 - Math.min(minPosting);
    let timeDiffDays = timeDiffMillis / 3600.0 / 24.0;

    timeDiffDays = 7.0;
    return totalCount / timeDiffDays;
}
