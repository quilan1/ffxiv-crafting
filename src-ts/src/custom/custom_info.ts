import RecStatistics, { RecStatisticsCollection } from "./rec_statistics.js";
import Statistics from "./statistics.js";
import Util from "../util/util.js";
import Api from "../util/api.js";

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
    output_info?: CustomInfoJson,
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

    static async fetch(searchFilter: string, dataCenter: string, countFn?: () => number, statusFn?: (_: string) => void): Promise<CustomInfo> {
        const lazyInfo = await this.apiPut(searchFilter, dataCenter);
        const id = lazyInfo.id;
        statusFn ??= () => {};
        countFn ??= () => 1;

        let info = null;
        while(info === null || info === undefined) {
            await Util.sleep(500);
            const lazyGetInfo = await this.apiGet(id);
            statusFn(lazyGetInfo.status ?? '');
            info = lazyGetInfo.output_info;
        }
        statusFn('');

        return this.customInfoFromJson(info, countFn());
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

    private static async apiGet(id: string): Promise<CustomInfoLazyJson> {
        return Api.call(this.api.get, { id });
    }

    private static async apiPut(searchFilter: string, dataCenter: string): Promise<CustomInfoLazyJson> {
        return Api.call(this.api.put, {}, { filters: searchFilter, data_center: dataCenter, retain_num_days: 14.0 });
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
            get: { endpoint: 'v1/custom', options: { method: 'GET', ...commonHeaders } },
            put: { endpoint: 'v1/custom', options: { method: 'PUT', ...commonHeaders } },
        }
    }
}
