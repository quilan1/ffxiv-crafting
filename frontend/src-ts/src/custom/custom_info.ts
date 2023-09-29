import RecStatistics, { RecStatisticsCollection } from "./rec_statistics.js";
import Statistics from "./statistics.js";
import { CancelData } from "./custom.js";
import UniversalisRequest from "./universalis_api.js";

////////////////////////////////////////////////////

export type Id = number;
export type IdChain = number[];

export type RecipeData = {
    item_id: Id,
    count: number,
}

export type Recipe = {
    inputs: RecipeData[],
    outputs: number,
}

export type Listing = {
    price: number,
    count: number,
    is_hq: boolean,
    world: string,
    name: string,
    posting: number,
}

export type ItemInfo = {
    item_id: number,
    name: string,
    listings: Listing[],
    history: Listing[],
    recipe?: Recipe,

    statistics: Statistics,
}

export type CustomInfoJson = {
    item_info: Record<Id, ItemInfo>,
    top_ids: number[],
    failure_ids: number[],
}

export type CustomInfoLazyJson = {
    id: string,
    status?: string,
    output_info?: CustomInfoJson,
}

export type RecipeJson = {
    top_ids: number[],
    item_info: Record<Id, ItemInfo>,
};

export type ListingJson = {
    id: string,
    status?: string,
    output_info?: {
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
        this.item_info = json.item_info;
        this.top_ids = json.top_ids;
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
        info.item_info = Object.fromEntries(Object.entries(info.item_info).map(([key, value]) => [Number.parseInt(key), value]));

        for (const [_, item] of Object.entries(info.item_info)) {
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
