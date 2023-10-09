import { ItemInfo } from "./items";
import { None, OptionType, optAdd, optMax, optMin, optSub } from "./option";
import { Statistics, statisticsOf } from "./statistics";
import { UniversalisInfo } from "./universalis_api"

type ItemStats = Record<number, Statistics>;
type ItemInfos = Record<number, ItemInfo>;
interface CraftInfo { key: number[], itemId: number, count: number };
type KeyedProfitStats = CraftInfo & ProfitStats;
interface TopProfitStats { top: KeyedProfitStats, children: KeyedProfitStats[] };

export interface RecursiveStats {
    itemStats: ItemStats,
    topProfitStats: TopProfitStats[],
}

interface ProfitStats {
    sell: OptionType<number>,
    buy: OptionType<number>,
    craft: OptionType<number>,
    profit: OptionType<number>,
}

interface ChildStats {
    itemId: number,
    count: number,
    stats: ProfitStats,
    childStats: ChildStats[],
}

export const allRecursiveStatsOf = (count: number, info: UniversalisInfo): RecursiveStats => {
    const allIds = allIdsOf(info);
    const itemStats = allIds
        .reduce<ItemStats>((prev, itemId) => ({ ...prev, [itemId]: statisticsOf(info.itemInfo[itemId]) }), {});
    const itemInfos = info.itemInfo;

    const topProfitStats = [];
    for (const itemId of info.topIds) {
        const childStats = recursiveStatsOf(itemId, count, itemInfos, itemStats);
        const [top, ...children] = flattenChildStats(childStats);
        topProfitStats.push({ top, children });
    }

    return { itemStats, topProfitStats };
}

const flattenChildStats = (childStats: ChildStats, parents?: number[]): KeyedProfitStats[] => {
    const key = (parents == undefined) ? [childStats.itemId] : [...parents, childStats.itemId];
    const results = [{ key, itemId: childStats.itemId, count: childStats.count, ...childStats.stats }];
    for (const child of childStats.childStats) {
        for (const flattenedStats of flattenChildStats(child, key)) {
            results.push(flattenedStats);
        }
    }
    return results;
}

const recursiveStatsOf = (itemId: number, count: number, itemInfos: ItemInfos, itemStats: ItemStats): ChildStats => {
    const recipe = itemInfos[itemId].recipe;
    const numOutputs = recipe?.outputs ?? 1;
    const numCrafts = Math.floor((count + numOutputs - 1) / numOutputs);
    const childStats = recipe?.inputs.map(input => recursiveStatsOf(input.itemId, input.count * numCrafts, itemInfos, itemStats)) ?? [];

    let craft = None<number>();
    for (const child of childStats) {
        const childBuy = child.stats.buy;
        const childCraft = child.stats.craft;
        const lowest = optMin(childBuy, childCraft);
        craft = optAdd(craft, lowest);
    }

    const _stats = itemStats[itemId];
    const sell = _stats.sellPrice.aq.map(v => v * numCrafts);
    const buy = _stats.buyPrice.aq.map(v => v * numCrafts);
    const profitBuy = buy.and(optSub(sell, buy));
    const profitCraft = craft.and(optSub(sell, craft));
    const profit = optMax(profitBuy, profitCraft).or(sell);

    const stats = {
        sell,
        buy,
        craft,
        profit,
    };

    return {
        itemId,
        count: numCrafts,
        stats,
        childStats,
    }
}

const allIdsOf = (info: UniversalisInfo, itemId?: number): number[] => {
    const childIds: number[] = (itemId === undefined)
        ? info.topIds
        : (info.itemInfo[itemId].recipe === undefined)
            ? []
            : info.itemInfo[itemId].recipe?.inputs.map(ingredient => ingredient.itemId)
            ?? [];

    const results = new Set<number>();
    for (const childId of childIds) {
        results.add(childId);
        for (const id of allIdsOf(info, childId)) {
            results.add(id);
        }
    }
    return [...results].toSorted((a, b) => a - b);
}
