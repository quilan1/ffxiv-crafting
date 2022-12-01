import RecStatistics, { RecStatisticsCollection } from "./rec_statistics.js";
import Statistics, { Quality } from "./statistics.js";
import Filters from "../filters.js";
import Util from "../util.js";

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
            info = await this.fetchDebug();
        } else {
            info = await this.fetchRaw(searchFilter, dataCenter);
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

    private static async fetchRaw(searchFilter: string, dataCenter: string): Promise<CustomInfoJson> {
        const encFilters = encodeURIComponent(searchFilter);

        try {
            let request = await Util.fetch(`v1/custom-filter?filters=${encFilters}&data_center=${dataCenter}`);
            let json = await request.json();
            return json;
        } catch (err) {
            console.error(err);
            throw err;
        }
    }

    private static async fetchDebug(): Promise<CustomInfoJson> {
        try {
            let request = await Util.fetch(`web/crafting-mats.json`);
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
}
