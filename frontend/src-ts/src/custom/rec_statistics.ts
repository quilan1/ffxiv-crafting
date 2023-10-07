import { Id, IdChain, ItemInfo, RecipeData } from "./custom_info";
import Util from "../util/util.js";
import { Quality } from "./statistics";

export enum RecStatisticsSkip {
    NoSkip,
    SkipChildren,
    SkipEverything,
}

export class RecStatisticsCollection {
    private readonly _entries: Record<number, RecStatistics>;

    constructor(entries?: Record<number, RecStatistics>) {
        this._entries = entries ?? {};
    }

    get entries(): [Id, RecStatistics][] {
        const list: [Id, RecStatistics][] = [];
        for (const [idStr, value] of Object.entries(this._entries)) {
            const id = Number.parseInt(idStr);
            list.push([id, value]);
        }
        return list;
    }

    get keys(): Id[] {
        return this.entries.map(([id, _]) => id);
    }

    get values(): RecStatistics[] {
        return this.entries.map(([_, stats]) => stats);
    }

    get(index: Id | IdChain): RecStatistics | undefined {
        if (typeof index === 'number') {
            return this._entries[index];
        }

        const [head, ...tail] = index;
        const entry = this.get(head);

        if (tail.length === 0) {
            return entry;
        }

        return entry?.inputs?.get(tail);
    }

    set(index: Id, value: RecStatistics) {
        this._entries[index] = value;
    }

    childChains(id?: IdChain): IdChain[] {
        id ??= [];

        let list: IdChain[] = [];
        for (const [_id, input] of this.entries) {
            const childHistory = [...id, _id];
            list.push(childHistory);

            if (input.inputs === undefined) {
                continue;
            }

            list = list.concat(input.inputs.childChains(childHistory));
        }

        return list;
    }

    allChainsOf(ids: Id[]): IdChain[] {
        let list: IdChain[] = [];
        for (const id of ids) {
            const childHistory = [id];
            list.push(childHistory);

            const input = this._entries[id];
            if (input.inputs === undefined) {
                continue;
            }

            list = list.concat(input.inputs.childChains(childHistory));
        }

        return list;
    }

    static filterChains(idChains: IdChain[], filter: (idChain: IdChain) => RecStatisticsSkip): IdChain[] {
        let retIdChains = [];
        let skipParent: IdChain | undefined = undefined;
        for (const idChain of idChains) {
            if (skipParent !== undefined) {
                const subArray = idChain.slice(0, skipParent.length);
                if (Util.equals(skipParent, subArray)) {
                    continue;
                }
                skipParent = undefined;
            }

            let result = filter(idChain);
            if (result === RecStatisticsSkip.SkipEverything) {
                skipParent = [...idChain];
                continue;
            } else if (result === RecStatisticsSkip.SkipChildren) {
                skipParent = [...idChain];
            }

            retIdChains.push(idChain);
        }

        return retIdChains;
    }
}

export default class RecStatistics {
    readonly medSellPrice?: number;
    readonly minBuyPrice?: number;
    readonly minCraftPrice?: number;
    readonly inputs?: RecStatisticsCollection;
    readonly item: ItemInfo;
    readonly count: number;

    private constructor(ingredient: RecipeData, allItems: Record<Id, ItemInfo>, multiplier: number) {
        this.item = allItems[ingredient.itemId];
        if (this.item === undefined) {
            throw new Error('Invalid item id');
        }

        const recMultiplier = ingredient.count * multiplier;
        this.count = recMultiplier;

        const maybeMul = (value: number | undefined) => value !== undefined ? value * recMultiplier : undefined;
        this.medSellPrice = maybeMul(qualityDefault(this.item.statistics.homeworldSellPrice));
        this.minBuyPrice = RecStatistics.calculateBuyCost(this.item, recMultiplier);

        const recipe = this.item.recipe;
        if (recipe === undefined) {
            return;
        }

        this.inputs = new RecStatisticsCollection();
        let minCraftPrice = undefined;
        for (const recipeData of recipe.inputs) {
            const inputItem = allItems[recipeData.itemId];
            if (inputItem === undefined) {
                continue;
            }

            const childRecMultiplier = Math.floor((recMultiplier + recipe.outputs - 1) / recipe.outputs);

            let childRecStatistics;
            try {
                childRecStatistics = new RecStatistics(recipeData, allItems, childRecMultiplier);
            } catch (_) {
                continue;
            }

            this.inputs.set(recipeData.itemId, childRecStatistics);
            if (childRecStatistics.minBuyPrice === undefined && childRecStatistics.minCraftPrice === undefined) {
                continue;
            }

            const childMinBuyPrice = childRecStatistics.minBuyPrice ?? Number.MAX_SAFE_INTEGER;
            const childMinCraftPrice = childRecStatistics.minCraftPrice ?? Number.MAX_SAFE_INTEGER;

            minCraftPrice ??= 0;
            minCraftPrice += Math.min(childMinBuyPrice, childMinCraftPrice);
        }

        this.minCraftPrice = minCraftPrice;
    }

    static from(id: Id, count: number, allItems: Record<Id, ItemInfo>): RecStatistics | undefined {
        try {
            return new RecStatistics({ itemId: id, count }, allItems, 1);
        } catch(_) {
            return undefined;
        }
    }

    get isBuyingCheaper() {
        const minBuy = this.minBuyPrice ?? Number.MAX_SAFE_INTEGER;
        const minCraft = this.minCraftPrice ?? Number.MAX_SAFE_INTEGER;
        return minBuy <= minCraft;
    }

    get profit() {
        const sellPrice = this.medSellPrice ?? 0;
        return sellPrice - this.buyCraftPrice;
    }

    get buyCraftPrice() {
        const craftPrice = this.minCraftPrice ?? Number.MAX_SAFE_INTEGER;
        const buyPrice = this.minBuyPrice ?? Number.MAX_SAFE_INTEGER;
        return Math.min(buyPrice, craftPrice);
    }

    private static calculateBuyCost(item: ItemInfo, count: number): number | undefined {
        if (item.listings.length === 0) {
            return undefined;
        }

        let cost = 0;
        let lastPosting = 0;
        for (const listing of item.listings) {
            const usedCount = Math.min(listing.count, count);
            lastPosting = listing.price;
            cost += listing.price * usedCount;
            count -= usedCount;

            if (count <= 0) {
                break;
            }
        }

        return cost + count * lastPosting;
    }
}

function qualityDefault<T>(quality?: Quality<T>): T | undefined {
    return quality?.hq ?? quality?.aq;
}
