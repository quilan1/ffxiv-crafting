import RecStatistics, { RecStatisticsCollection } from "./rec_statistics.js";
import Statistics from "./statistics.js";
import { CancelData } from "./custom.js";
import UniversalisRequest from "./universalis_api.js";

////////////////////////////////////////////////////

export type Id = number;
export type IdChain = number[];

export type RecipeData = {
    itemId: Id,
    count: number,
}

export type Recipe = {
    inputs: RecipeData[],
    outputs: number,
}

export type Listing = {
    price: number,
    count: number,
    isHq: boolean,
    world: string,
    name: string,
    posting: number,
}

export type ItemInfo = {
    itemId: number,
    name: string,
    listings: Listing[],
    history: Listing[],
    recipe?: Recipe,

    statistics: Statistics,
}

export type CustomInfoJson = {
    itemInfo: Record<Id, ItemInfo>,
    topIds: number[],
    failureIds: number[],
}

export type CustomInfoLazyJson = {
    id: string,
    status?: string,
    outputInfo?: CustomInfoJson,
}

export type RecipeJson = {
    topIds: number[],
    itemInfo: Record<Id, ItemInfo>,
};

export type ListingJson = {
    id: string,
    status?: string,
    outputInfo?: {
        failures: number[],
        listings: Record<number, Listing[]>,
    }
}

export type ListingOutputJson = {
    failures: number[],
    listings: Record<number, Listing[]>,
}

export class CancelError extends Error {
    constructor(message?: string, options?: ErrorOptions) {
        super(message, options);
        this.name = 'CancelError';
    }
}

////////////////////////////////////////////////////

export default class CustomInfo {
    readonly item_info: Record<Id, ItemInfo>;
    readonly top_ids: number[];
    readonly rec_statistics: RecStatisticsCollection;

    constructor(json: CustomInfoJson, count: number) {
        this.item_info = json.itemInfo;
        this.top_ids = json.topIds;
        this.rec_statistics = new RecStatisticsCollection();
        this.calcRecStatistics(count);
    }

    static async fetch(searchFilter: string, dataCenter: string, countFn?: () => number, statusFn?: (_: string) => void, cancelData?: CancelData): Promise<CustomInfo> {
        countFn ??= () => 1;

        const universalis_request = new UniversalisRequest(searchFilter, dataCenter);
        if (statusFn !== undefined) universalis_request.setStatusFn(statusFn);
        if (cancelData !== undefined) universalis_request.setCancelData(cancelData);

        let recipeInfo = await universalis_request.fetch();
        return this.customInfoFromJson(recipeInfo, countFn());
    }

    private static customInfoFromJson(info: CustomInfoJson, count: number): CustomInfo {
        info.itemInfo = Object.fromEntries(Object.entries(info.itemInfo).map(([key, value]) => [Number.parseInt(key), value]));

        for (const [_, item] of Object.entries(info.itemInfo)) {
            if (item.recipe === null) {
                delete item.recipe;
            }
            item.statistics = new Statistics(item);
        }

        return new CustomInfo(info, count);
    }

    ////////////////////////////////////////////////////

    calcRecStatistics(count: number) {
        for (const id of this.top_ids) {
            let stats = RecStatistics.from(id, count, this.item_info);
            if (stats !== undefined) {
                this.rec_statistics.set(id, stats);
            }
        }
    }
}
