import RecStatistics, { RecStatisticsCollection } from "./rec_statistics.js";
import Statistics from "./statistics.js";
import Util from "../util/util.js";
import Api from "../util/api.js";
import { CancelData } from "./custom.js";

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

    static async fetchDebug(count: number): Promise<CustomInfo> {
        return this.customInfoFromJson(await this.apiGetDebug(), count);
    }

    static async fetch(searchFilter: string, dataCenter: string, countFn?: () => number, statusFn?: (_: string) => void, cancelData?: CancelData): Promise<CustomInfo> {
        const [recipe, listingId, historyId] = await Promise.all([
            this.apiRecipeGet(searchFilter),
            this.apiListingsPut(searchFilter, dataCenter),
            this.apiHistoryPut(searchFilter, dataCenter)
        ]);
        statusFn ??= () => { };
        countFn ??= () => 1;

        const checkCancel = async (): Promise<void> => {
            if (cancelData?.cancelled !== true)
                return;

            try { await this.apiCancel(listingId); }
            catch { }

            try { await this.apiCancel(historyId); }
            catch { }

            statusFn!('');
            throw new CancelError("Cancelled transaction");
        }

        let listingInfo = null, historyInfo = null;
        while (listingInfo == null || historyInfo == null) {
            await checkCancel();
            await Util.sleep(500);

            const listingPromise = listingInfo ? null : this.apiListingGet(listingId);
            const historyPromise = historyInfo ? null : this.apiHistoryGet(historyId);

            let listingStatus = 'Done', historyStatus = 'Done';
            if (listingPromise != null) {
                const info: ListingJson = await listingPromise;
                listingInfo = info.output_info;
                listingStatus = info?.status ?? '';
            }
            if (historyPromise != null) {
                const info: ListingJson = await historyPromise;
                historyInfo = info.output_info;
                historyStatus = info?.status ?? '';
            }

            statusFn(`Listings: ${listingStatus}\nHistories: ${historyStatus}`);
        }
        statusFn('');

        let info = recipe;
        for (const [id, item] of Object.entries(recipe.item_info)) {
            item.listings = listingInfo.listings[id as any] ?? [];
            item.history = historyInfo?.listings[id as any] ?? [];
        }

        return this.customInfoFromJson(info as CustomInfoJson, countFn());
    }

    static async apiCancel(id: string): Promise<void> {
        await Api.call(this.api.cancel.put(id));
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

    private static apiRecipeGet(searchFilter: string): Promise<RecipeJson> {
        return Api.call(this.api.recipe.get, { filters: searchFilter });
    }

    private static async apiGenListingsPut(api: any, searchFilter: string, dataCenter: string): Promise<string> {
        return Api.call(api, {}, { filters: searchFilter, data_center: dataCenter, retain_num_days: 14.0 });
    }

    private static async apiListingsPut(searchFilter: string, dataCenter: string): Promise<string> {
        return this.apiGenListingsPut(this.api.listings.put, searchFilter, dataCenter);
    }

    private static async apiHistoryPut(searchFilter: string, dataCenter: string): Promise<string> {
        return this.apiGenListingsPut(this.api.history.put, searchFilter, dataCenter);
    }

    private static async apiListingGet(id: string): Promise<ListingJson> {
        return Api.call(this.api.listings.get(id));
    }

    private static async apiHistoryGet(id: string): Promise<ListingJson> {
        return Api.call(this.api.history.get(id));
    }

    ////////////////////////////////////////////////////

    private static async apiGetDebug(): Promise<CustomInfoJson> {
        try {
            let request = await Api.getPage(`crafting-mats.json`);
            let json = await request.json();
            return json;
        } catch (err) {
            console.error(err);
            throw err;
        }
    }

    calcRecStatistics(count: number) {
        for (const id of this.top_ids) {
            let stats = RecStatistics.from(id, count, this.item_info);
            if (stats !== undefined) {
                this.rec_statistics.set(id, stats);
            }
        }
    }

    private static get api() {
        const commonHeaders = {
            headers: { 'Content-Type': 'application/json' },
        };

        return {
            listings: {
                get: (id: string) => ({ endpoint: `v1/market/${id}`, options: { method: 'GET', ...commonHeaders } }),
                put: { endpoint: 'v1/market/listings', options: { method: 'PUT', ...commonHeaders } },
            },
            history: {
                get: (id: string) => ({ endpoint: `v1/market/${id}`, options: { method: 'GET', ...commonHeaders } }),
                put: { endpoint: 'v1/market/history', options: { method: 'PUT', ...commonHeaders } },
            },
            recipe: {
                get: { endpoint: 'v1/recipe', options: { method: 'GET', ...commonHeaders } },
            },
            cancel: {
                put: (id: string) => ({ endpoint: `v1/market/${id}/cancel`, options: { method: 'PUT', ...commonHeaders } }),
            }
        }
    }
}
