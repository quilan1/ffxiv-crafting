/* eslint-disable @typescript-eslint/no-unused-vars */
import { KeyedTableRow } from "../(query)/(table)/table";
import { None, OptionType, Some } from "./option";
import { UniversalisInfo } from "./universalis_api"

type ItemCounts = Record<number, number|undefined>;
interface ListingType<T> { listings: T, history: T };

export default class UniversalisAnalysis {
    private info: UniversalisInfo;

    constructor(info: UniversalisInfo) {
        this.info = info;
    }

    generateTableData(count: number, limit: number): KeyedTableRow[] {
        const results = [];
        for (const itemId of this.info.topIds) {
            const itemInfo = this.info.itemInfo[itemId];
            const quantity = count > 1 ? `${count}x ` : '';
            results.push({
                key: `${itemId}`,
                row: {
                    name: `${quantity}${itemInfo.name}`,
                    checked: false,
                    hidden: false,
                    perDay: '-',
                    perWeek: '-',
                    perBiWeek: '-',
                    count: '-',
                    sell: '-',
                    buy: '-',
                    craft: '-',
                    profit: '-',
                }
            });
        }
        return results;
    }

    private itemCounts(count: number): Record<number, ItemCounts> {
        const totalCounts: Record<number, ItemCounts> = {};
        for (const itemId of this.info.topIds) {
            totalCounts[itemId] = this.itemCountsForId(itemId, count);
        }
        return totalCounts;
    }

    private itemCountsForId(itemId: number, count: number): ItemCounts {
        const totalCounts: ItemCounts = {};

        const info = this.info;
        function recurseCounts(itemId: number, multiplier: number) {
            totalCounts[itemId] = (totalCounts[itemId] ?? 0) + multiplier;

            const recipe = info.itemInfo[itemId].recipe;
            if (!recipe) return;

            const numCrafts = Math.floor((multiplier + recipe.outputs - 1) / recipe.outputs);
            for (const { itemId, count } of recipe.inputs) {
                recurseCounts(itemId, count * numCrafts);
            }
        }

        recurseCounts(itemId, count);
        return totalCounts;
    }
}

const medianOf = (values: number[]): OptionType<number> => {
    if (values.length == 0)
        return None();

    const midIndex = values.length / 2;
    const a = values[midIndex];
    if (values.length % 2 > 0)
        return Some(a);

    const b = values[midIndex + 1];
    return Some((a + b) / 2);
}

const meanOf = (values: number[]): OptionType<number> => {
    if (values.length == 0)
        return None();

    return Some(values.reduce((a, b) => a + b) / values.length);
}

const stripOutliers = (values: number[], numStdDev: number): number[] => {
    const _average = meanOf(values);
    if (!_average.is_some())
        return [];

    const mean = _average.unwrap();
    const totalVariance = values
        .map(x => (x - mean) ** 2)
        .reduce((a, b) => (a + b));
    const variance = totalVariance / values.length;
    const stdDev = Math.sqrt(variance);

    return values.filter(x => Math.abs((x - mean) / stdDev) <= numStdDev);
}

function postingToDays(posting: number) {
    return (Date.now() / 1000.0 - posting) / 3600.0 / 24.0;
}
