import { ItemInfo } from "./items";
import { None, OptionType, Some, optAdd, optMax, optMin, optSub } from "./option";
import { Quality, Statistics, preferHq, statisticsOf } from "./statistics";
import { UniversalisInfo } from "./universalis_api"
import Util from "./util";

type ItemStats = Record<number, Statistics>;
type ItemInfos = Record<number, ItemInfo>;
interface CraftInfo { key: number[], itemId: number, count: number, hasChildren: boolean };
interface KeyedProfitStats extends CraftInfo, ProfitStats { };
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

export const allRecursiveStatsOfAsync = (count: number, isHq: boolean, info: UniversalisInfo): Promise<RecursiveStats> => {
    return new Promise((resolve, _reject) => {
        const worker = new Worker(new URL("./analysis-worker", import.meta.url));
        worker.postMessage({ count, isHq, info });
        worker.onmessage = (e: MessageEvent<RecursiveStats>) => {
            const stats = e.data;
            reattachOptions(stats);
            resolve(stats);
        };
    });
}

export const allRecursiveStatsOf = (count: number, isHq: boolean, info: UniversalisInfo): RecursiveStats => {
    const allIds = allIdsOf(info);
    const maxCounts = maxCountsOf(info.itemInfo, count);
    const itemStats = allIds
        .reduce<ItemStats>((prev, itemId) => ({ ...prev, [itemId]: statisticsOf(info.itemInfo[itemId], maxCounts[itemId]) }), {});
    const itemInfos = info.itemInfo;

    const topProfitStats = [];
    for (const itemId of info.topIds) {
        const childStats = recursiveStatsOf(itemId, count, isHq, isHq, itemInfos, itemStats);
        const [top, ...children] = flattenChildStats(childStats);
        topProfitStats.push({ top, children });
    }

    return { itemStats, topProfitStats };
}

const flattenChildStats = (childStats: ChildStats, parents?: number[]): KeyedProfitStats[] => {
    const key = (parents == undefined) ? [childStats.itemId] : [...parents, childStats.itemId];
    const results = [{
        key,
        itemId: childStats.itemId,
        count: childStats.count,
        hasChildren: childStats.childStats.length > 0,
        ...childStats.stats
    }];
    for (const child of childStats.childStats) {
        for (const flattenedStats of flattenChildStats(child, key)) {
            results.push(flattenedStats);
        }
    }
    return results;
}

const recursiveStatsOf = (itemId: number, count: number, isHq: boolean, isTop: boolean, itemInfos: ItemInfos, itemStats: ItemStats): ChildStats => {
    const recipe = itemInfos[itemId].recipe;
    const numOutputs = recipe?.outputs ?? 1;
    const numCrafts = Math.floor((count + numOutputs - 1) / numOutputs);
    const _count = numCrafts * numOutputs;
    const childStats = recipe?.inputs.map(input =>
        recursiveStatsOf(input.itemId, input.count * numCrafts, isHq, false, itemInfos, itemStats)
    ) ?? [];

    let craft = None<number>();
    for (const child of childStats) {
        const childBuy = child.stats.buy;
        const childCraft = child.stats.craft;
        const lowest = optMin(childBuy, childCraft);
        craft = optAdd(craft, lowest);
    }

    const _stats = itemStats[itemId];
    const sellPrice = preferHq(_stats.sellPrice, isHq, isTop && craft.is_some());
    const buyPrice = preferHq(_stats.buyPrice, isHq, isTop && craft.is_some());
    const sell = sellPrice.map(v => v * _count);
    const buy = buyPrice.map(v => v * _count);
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
        count: _count,
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

const maxCountsOf = (info: Record<number, ItemInfo>, count: number, itemId?: number): Record<number, number> => {
    const maxCounts: Record<number, number | undefined> = {};

    if (itemId == undefined) {
        for (const itemId of Util.keysOf(info)) {
            const childMaxCounts = maxCountsOf(info, count, itemId);
            for (const [childItemId, count] of Util.entriesOf(childMaxCounts)) {
                maxCounts[childItemId] = Math.max(maxCounts[childItemId] ?? 0, count);
            }
        }
    } else {
        const item = info[itemId];
        const numOutputs = item.recipe?.outputs ?? 1;
        const numCrafts = Math.floor((count + numOutputs - 1) / numOutputs);
        const _count = numCrafts * numOutputs;
        maxCounts[itemId] = (maxCounts[itemId] ?? 0) + _count;

        for (const ingredient of item.recipe?.inputs ?? []) {
            const childMaxCounts = maxCountsOf(info, numCrafts * ingredient.count, ingredient.itemId);
            for (const [childItemId, count] of Util.entriesOf(childMaxCounts)) {
                maxCounts[childItemId] = (maxCounts[childItemId] ?? 0) + count;
            }
        }
    }

    return maxCounts as Record<number, number>;
}

const reattachOptions = (stats: RecursiveStats) => {
    const optProto = Object.getPrototypeOf(Some(0)) as object;

    const reattach = (opt: OptionType<number>) => {
        Object.setPrototypeOf(opt, optProto);
    }

    const reattachQuality = (quality: Quality<number>) => {
        reattach(quality.hq);
        reattach(quality.nq);
        reattach(quality.aq);
    }

    for (const key of Util.keysOf(stats.itemStats)) {
        const val = stats.itemStats[key];
        reattachQuality(val.buyPrice);
        reattachQuality(val.sellCount);
        reattachQuality(val.sellPrice);
        reattachQuality(val.velocityDay);
        reattachQuality(val.velocityWeek);
        reattachQuality(val.velocityWeeks);
    }

    for (const { top, children } of stats.topProfitStats) {
        for (const profit of [top, ...children]) {
            reattach(profit.buy);
            reattach(profit.sell);
            reattach(profit.craft);
            reattach(profit.profit);
        }
    }
}