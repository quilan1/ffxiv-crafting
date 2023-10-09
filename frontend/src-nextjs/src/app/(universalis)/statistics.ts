import { ItemInfo, Listing } from "./items";
import { None, OptionType, Some } from "./option";

const HOMEWORLD = "Seraph";

export interface Quality<T> {
    hq: OptionType<T>,
    nq: OptionType<T>,
    aq: OptionType<T>,
}

export interface Statistics {
    buyPriceHomeworld: Quality<number>,
    buyPrice: Quality<number>,
    sellPrice: Quality<number>,
    sellCount: Quality<number>,
    velocityDay: Quality<number>,
    velocityWeek: Quality<number>,
    velocityWeeks: Quality<number>,
}

export const statisticsOf = (itemInfo: ItemInfo): Statistics => {
    const isHomeworld = (listing: Listing) => listing.world === undefined || listing.world === HOMEWORLD;
    const toPrice = (listing: Listing) => listing.price;
    const toCount = (listing: Listing) => listing.count;

    const isWithinDay = isWithinDaysFn(1.0);
    const isWithinWeek = isWithinDaysFn(7.0);
    const isWithinWeeks = isWithinDaysFn(14.0);
    const stripOutliersOf = stripOutliersOfFn(2.0);
    const medianOf = medianOfFn(0.3);

    const buyPrice = quality(itemInfo.listings, _ => _.map(toPrice).apply(stripOutliersOf).apply(medianOf));
    const buyPriceHomeworld = quality(itemInfo.listings, _ => _.filter(isHomeworld).map(toPrice).apply(stripOutliersOf).apply(medianOf));

    const sellCount = quality(itemInfo.history, _ => _.filter(isHomeworld).filter(isWithinWeek).map(toCount).apply(meanOf));
    const sellPrice = quality(itemInfo.history, _ => _.filter(isHomeworld).map(toPrice).apply(stripOutliersOf).apply(meanOf));
    const velocityDay = quality(itemInfo.history, _ => _.filter(isHomeworld).filter(isWithinDay).apply(velocity));
    const velocityWeek = quality(itemInfo.history, _ => _.filter(isHomeworld).filter(isWithinWeek).apply(velocity));
    const velocityWeeks = quality(itemInfo.history, _ => _.filter(isHomeworld).filter(isWithinWeeks).apply(velocity));

    return { buyPrice, buyPriceHomeworld, sellCount, sellPrice, velocityDay, velocityWeek, velocityWeeks };
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
        if (!_mean.is_some()) return new SimpleArray([]);

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

function velocity(listings: Listing[]): OptionType<number> {
    if (listings.length == 0) return None();
    const totalCount = listings.reduce((prev, item) => prev + item.count, 0);
    const maxDaysSince = Math.max(...listings.map(item => item.daysSince));
    return Some(totalCount / maxDaysSince);
}
