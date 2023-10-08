import { ItemInfo, Listing } from "./custom_info.js";

const HOMEWORLD = "Seraph";

export type Quality<T> = {
    aq: T,
    nq: T,
    hq?: T,
}

export default class Statistics {
    readonly minBuyPrice?: Quality<number>;
    readonly homeworldAvgSellCount?: Quality<number>;
    readonly homeworldSellPrice?: Quality<number>;
    readonly homeworldMedBuyPrice?: Quality<number>;
    readonly homeworldVelocityDay?: Quality<number>;
    readonly homeworldVelocityWeek?: Quality<number>;
    readonly homeworldVelocityWeeks?: Quality<number>;

    constructor(itemInfo: ItemInfo) {
        const isHomeworld = (listing: Listing) => listing.world === '' || listing.world === HOMEWORLD;
        const isWithinDays = (days: number) => (listing: Listing) => listing.daysSince <= days;
        const toIdent = (listing: Listing) => listing;
        const toPrice = (listing: Listing) => listing.price;
        const toCount = (listing: Listing) => listing.count;
        const toPostingPrice = (listing: Listing) => { return { days: listing.daysSince, value: listing.price } };
        const average = (values: number[]) => (values.length == 0) ? 0 : values.reduce((a, b) => a + b, 0) / values.length;
        const median = (values: number[]) => { values.sort((a, b) => a - b); return values[(values.length / 3) | 0]; };
        const min = (values: number[]) => values.reduce((prev: number | undefined, cur: number) => (prev === undefined || cur < prev) ? cur : prev, undefined) as number;

        this.minBuyPrice = generateQuality(itemInfo.name, itemInfo.listings, [], toPrice, min);
        this.homeworldMedBuyPrice = generateQuality(itemInfo.name, itemInfo.listings, [isHomeworld], toPrice, median);

        this.homeworldAvgSellCount = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld, isWithinDays(7.0)], toCount, average);
        this.homeworldSellPrice = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld], toPostingPrice, weightedTimeAverage(1.0, 7.0));
        this.homeworldVelocityDay = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld, isWithinDays(1.0)], toIdent, calculateVelocity);
        this.homeworldVelocityWeek = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld, isWithinDays(7.0)], toIdent, calculateVelocity);
        this.homeworldVelocityWeeks = generateQuality(itemInfo.name, itemInfo.history, [isHomeworld, isWithinDays(14.0)], toIdent, calculateVelocity);
    }
}

function generateQuality<T, O>(name: string, listings: Listing[], filter_fns: ((item: Listing) => boolean)[], map_fn: (item: Listing) => T, reduce_fn: (items: T[]) => O): Quality<O> | undefined {
    if (listings.length === 0) return undefined;

    const isHq = (item: Listing) => item.isHq;
    const isNq = (item: Listing) => !item.isHq;

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

function calculateVelocity(listings: Listing[]): number {
    const totalCount = listings.reduce((prev, item) => prev + item.count, 0);
    const maxDaysSince = Math.max(...listings.map(item => item.daysSince));
    return totalCount / maxDaysSince;
}

// Creates a function that does a weighted-average, with a normal distribution
function weightedTimeAverage(meanDays: number, stdDevDays: number) {
    return (listingInfos: { days: number, value: number }[]) => {
        let numerator = 0, denomenator = 0;
        for (const { days, value } of listingInfos) {
            let coefficient = Math.exp(-(days - meanDays) * (days - meanDays) / (2 * stdDevDays * stdDevDays));
            numerator += coefficient * value;
            denomenator += coefficient;
        }

        return Math.round(numerator / denomenator);
    }
}
