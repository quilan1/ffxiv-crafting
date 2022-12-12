import RecStatistics, { RecStatisticsCollection } from "./rec_statistics.js";
import Statistics from "./statistics.js";
import Util from "../util.js";
import Api from "../api.js";

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
}

export type CustomInfoLazyJson = {
    id: string,
    status?: string,
    output?: CustomInfoJson,
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

    static async fetch(searchFilter: string, count: number, dataCenter: string, isDebug?: boolean): Promise<CustomInfo> {
        let info;
        if (isDebug === true) {
            info = await this.apiGetDebug();
        } else {
            info = await this.apiGet(searchFilter, dataCenter);
        }

        info.item_info = Object.fromEntries(Object.entries(info.item_info).map(([key, value]) => [Number.parseInt(key), value]));

        for (const [_, item] of Object.entries(info.item_info)) {
            if (item.recipe === null) {
                delete item.recipe;
            }
            item.statistics = new Statistics(item);
        }

        return new CustomInfo(info, count);
    }

    static async fetchLazy(searchFilter: string, count: number, dataCenter: string, statusFn?: (_: string) => void): Promise<CustomInfo> {
        const lazyInfo = await this.apiPutLazy(searchFilter, dataCenter);
        const id = lazyInfo.id;
        statusFn ??= () => {};

        let info = null;
        while(info === null || info === undefined) {
            await Util.sleep(500);
            const lazyGetInfo = await this.apiGetLazy(id);
            statusFn(lazyGetInfo.status ?? '');
            info = lazyGetInfo.output;
        }
        statusFn('');

        info.item_info = Object.fromEntries(Object.entries(info.item_info).map(([key, value]) => [Number.parseInt(key), value]));

        for (const [_, item] of Object.entries(info.item_info)) {
            if (item.recipe === null) {
                delete item.recipe;
            }
            item.statistics = new Statistics(item);
        }

        return new CustomInfo(info, count);
    }

    private static async apiGet(searchFilter: string, dataCenter: string): Promise<CustomInfoJson> {
        return Api.call(this.api.filters.get, { filters: searchFilter, data_center: dataCenter, retain_num_days: 7.0 });
    }

    private static async apiGetLazy(id: string): Promise<CustomInfoLazyJson> {
        return Api.call(this.api.lazy.get, { id });
    }

    private static async apiPutLazy(searchFilter: string, dataCenter: string): Promise<CustomInfoLazyJson> {
        return Api.call(this.api.lazy.put, {}, { filters: searchFilter, data_center: dataCenter, retain_num_days: 14.0 });
    }

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
            get filters() {
                return {
                    get: { endpoint: 'v1/custom-filter', options: { method: 'GET', ...commonHeaders } },
                }
            },
            get lazy() {
                return {
                    get: { endpoint: 'v1/custom', options: { method: 'GET', ...commonHeaders } },
                    put: { endpoint: 'v1/custom', options: { method: 'PUT', ...commonHeaders } },
                }
            }
        }
    }
}
