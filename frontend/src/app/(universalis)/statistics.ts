import { ItemInfo, Listing } from "./items";
import { None, OptionType, Some } from "../(util)/option";

export interface Quality<T> {
    hq: OptionType<T>,
    nq: OptionType<T>,
    aq: OptionType<T>,
}

export interface Statistics {
    buyPrice: Quality<number>,
    numListings: Quality<number>,
    totalNumListings: Quality<number>,
    sellPrice: Quality<number>,
    sellCount: Quality<number>,
    velocityWeeks: Quality<number>,
}

export const statisticsOf = (itemInfo: ItemInfo, count: number, homeworld: string): Statistics => {
    const isHomeworld = (listing: Listing) => listing.world === undefined || listing.world === homeworld;
    const toPrice = (listing: Listing) => listing.price;
    const toCount = (listing: Listing) => listing.count;

    const length = (values: number[]) => (!values.length) ? None() : Some(values.length);
    const isWithinWeeks = isWithinDaysFn(14.0);
    const stripOutliersOf = stripOutliersOfFn(2.0);
    const _medianOf = medianOfFn(0.3);
    const minForCountOf = minForCountOfFn(count);
    const velocityOfWeeks = velocityFn(14.0);
    const homeworldWeeks = (_: Listing) => isHomeworld(_) && isWithinWeeks(_);

    const buyPrice = quality(itemInfo.listings, _ => _.apply(minForCountOf));
    const numListings = quality(itemInfo.listings, _ => _.filter(isHomeworld).map(toPrice).apply(stripOutliersOf).apply(length));
    const totalNumListings = quality(itemInfo.listings, _ => _.filter(isHomeworld).map(toPrice).apply(length));

    const sellCount = quality(itemInfo.history, _ => _.filter(homeworldWeeks).map(toCount).apply(meanOf));
    const sellPrice = quality(itemInfo.history, _ => _.filter(homeworldWeeks).map(toPrice).apply(stripOutliersOf).apply(meanOf));
    const velocityWeeks = quality(itemInfo.history, _ => _.filter(homeworldWeeks).apply(velocityOfWeeks));

    return { buyPrice, numListings, totalNumListings, sellCount, sellPrice, velocityWeeks };
}

function quality<T>(listings: Listing[], fn: (listings: SimpleArray<Listing>) => OptionType<T>) {
    const hq = listings.filter(listing => listing.isHq);
    const nq = listings.filter(listing => !listing.isHq);
    return {
        hq: fn(new SimpleArray(hq)),
        nq: fn(new SimpleArray(nq)),
        aq: fn(new SimpleArray(listings)),
    }
}

export const maxVelocityOf = (stats: Statistics, isHq: boolean) => {

    const arr = [
        selectQuality(stats.velocityWeeks, isHq).unwrapOr(0)
    ].filter(v => v > 0);

    if (arr.length == 0) return 0;
    return arr.reduce((a, b) => Math.max(a, b));
}

export function selectQuality<T>(q: Quality<T>, isHq: boolean) {
    return isHq ? q.hq : q.aq;
}

export function preferHq<T>(quality: Quality<T>, isHq: boolean, reqHq: boolean) {
    const hqOpt = isHq ? quality.hq : None();
    return reqHq ? hqOpt : hqOpt.or(quality.aq);
}

class SimpleArray<T> {
    values: T[];
    constructor(values: T[]) {
        this.values = values;
    }

    filter(fn: (elem: T) => boolean): SimpleArray<T> {
        return new SimpleArray(this.values.filter(fn));
    }

    map<U>(fn: (elem: T) => U): SimpleArray<U> {
        return new SimpleArray(this.values.map(fn));
    }

    apply<U = T>(fn: (elems: T[]) => U) {
        return fn(this.values);
    }
}

const stripOutliersOfFn = (numStdDev: number) => {
    return (values: number[]) => {
        const _mean = meanOf(values);
        if (!_mean.isSome()) return new SimpleArray([]);

        const mean = _mean.unwrap();
        const totalVariance = values
            .map(x => (x - mean) ** 2)
            .reduce((a, b) => (a + b));
        const variance = totalVariance / values.length;
        const stdDev = Math.sqrt(variance);

        return new SimpleArray(values.filter(x => Math.abs((x - mean) / stdDev) <= numStdDev));
    }
}

const isWithinDaysFn = (days: number) => {
    return (listing: Listing) => listing.daysSince <= days;
}

const medianOfFn = (ratio = .5) => {
    return (values: number[]): OptionType<number> => {
        if (values.length == 0) return None();
        const index = Math.floor(values.length * ratio);
        const a = values[index];
        const b = (index < values.length) ? values[index + 1] : a;
        return Some((a + b) / 2.0);
    }
}

const meanOf = (values: number[]): OptionType<number> => {
    if (values.length == 0) return None();
    return Some(values.reduce((a, b) => a + b) / values.length);
}

const minForCountOfFn = (count: number) => {
    return (listings: Listing[]): OptionType<number> => {
        if (listings.length == 0) return None();
        const sortedListings = listings.toSorted((a, b) => a.price - b.price);
        let totalPrice = 0, remCount = count;
        for (const listing of sortedListings) {
            if (remCount == 0) break;
            const _count = Math.min(remCount, listing.count);
            totalPrice += listing.price * _count;
            remCount -= _count;
        }
        return Some(totalPrice / (count - remCount));
    }
}

function velocityFn(days: number, useLastSeen?: boolean) {
    return (listings: Listing[]): OptionType<number> => {
        if (listings.length == 0) return None();
        const totalCount = listings.reduce((prev, item) => prev + item.count, 0);
        const maxDaysSince = useLastSeen
            ? Math.max(...listings.map(item => item.daysSince))
            : days;
        return Some(totalCount / maxDaysSince);
    }
}
