import { ItemInfo, RecipeData } from "./custom_info";
import Util from "../util.js";
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

    get entries(): [number, RecStatistics][] {
        const list: [number, RecStatistics][] = [];
        for (const [idStr, value] of Object.entries(this._entries)) {
            const id = Number.parseInt(idStr);
            list.push([id, value]);
        }
        return list;
    }

    get keys(): number[] {
        return this.entries.map(([id, _]) => id);
    }

    get values(): RecStatistics[] {
        return this.entries.map(([_, stats]) => stats);
    }

    get(index: number | number[]): RecStatistics | undefined {
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

    set(index: number, value: RecStatistics) {
        this._entries[index] = value;
    }

    allChains(history?: number[]): number[][] {
        if (history === undefined) {
            history = [];
        }

        let list: number[][] = [];
        for (const [id, input] of this.entries) {
            const childHistory = [...history, id];
            list.push(childHistory);

            if (input.inputs === undefined) {
                continue;
            }

            list = list.concat(input.inputs.allChains(childHistory));
        }

        return list;
    }

    allChainsOf(ids: number[]): number[][] {
        let list: number[][] = [];
        for (const id of ids) {
            const childHistory = [id];
            list.push(childHistory);

            const input = this._entries[id];
            if (input.inputs === undefined) {
                continue;
            }

            list = list.concat(input.inputs.allChains(childHistory));
        }

        return list;
    }

    static filterChains(idChains: number[][], filter: (idChain: number[]) => RecStatisticsSkip): number[][] {
        let retIdChains = [];
        let skipParent: number[] | undefined = undefined;
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

    private constructor(ingredient: RecipeData, allItems: Record<number, ItemInfo>, multiplier: number) {
        this.item = allItems[ingredient.item_id];
        if (this.item === undefined) {
            throw new Error('Invalid item id');
        }

        const recMultiplier = ingredient.count * multiplier;
        this.count = recMultiplier;

        const maybeMul = (value: number | undefined) => value !== undefined ? value * recMultiplier : undefined;
        this.medSellPrice = maybeMul(qualityDefault(this.item.statistics.homeworldMedSellPrice));
        this.minBuyPrice = RecStatistics.calculateBuyCost(this.item, recMultiplier);

        const recipe = this.item.recipe;
        if (recipe === undefined) {
            return;
        }

        this.inputs = new RecStatisticsCollection();
        let minCraftPrice = undefined;
        for (const recipeData of recipe.inputs) {
            const inputItem = allItems[recipeData.item_id];
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

            this.inputs.set(recipeData.item_id, childRecStatistics);
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

    static from(id: number, count: number, allItems: Record<number, ItemInfo>): RecStatistics | undefined {
        try {
            return new RecStatistics({ item_id: id, count }, allItems, 1);
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
