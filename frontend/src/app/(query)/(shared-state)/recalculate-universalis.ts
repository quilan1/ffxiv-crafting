import { RecursiveStats } from "@/app/(universalis)/analysis";
import { KeyedTableRow } from "../(table)/table";
import { allRecursiveStatsOfAsync } from "@/app/(universalis)/analysis-async";
import { optSub } from "@/app/(util)/option";
import { maxVelocityOf, selectQuality } from "@/app/(universalis)/statistics";
import { QueryShared } from "./query-shared";

export enum ChangedState {
    COUNT,
    LIMIT,
    MIN_VELOCITY,
    IS_HQ,
    UNIVERSALIS_INFO,
}

export const recalculateUniversalis = async (data: QueryShared, changedStates: Set<ChangedState>) => {
    if (data.universalisInfo === undefined) {
        return;
    }

    if (changedStates.has(ChangedState.UNIVERSALIS_INFO)) {
        data.checkedKeys = new Set();
    }

    if (changedStates.has(ChangedState.UNIVERSALIS_INFO)
        || changedStates.has(ChangedState.COUNT)
        || changedStates.has(ChangedState.IS_HQ)) {
        await recalculateRecStatistics(data);
    }

    generateTableData(data);

    if (changedStates.has(ChangedState.UNIVERSALIS_INFO)
        || changedStates.has(ChangedState.IS_HQ)) {
        data.hiddenKeys = new Set<string>();
        for (const { key, row } of data.tableRows ?? []) {
            if (row.buy.unwrapOr(Number.MAX_SAFE_INTEGER) < row.craft.unwrapOr(Number.MIN_SAFE_INTEGER)) {
                data.hiddenKeys.add(key);
            }
        }
    }
}

const recalculateRecStatistics = async (data: QueryShared): Promise<RecursiveStats | undefined> => {
    if (data.universalisInfo === undefined) {
        data.recursiveStats = undefined;
        return undefined;
    }
    data.recursiveStats = await allRecursiveStatsOfAsync(data.count, data.isHq, data.universalisInfo, data.homeworld)
}

function generateTableData(data: QueryShared): KeyedTableRow[] | undefined {
    if (data.universalisInfo === undefined || data.recursiveStats === undefined) {
        data.tableRows = undefined;
        return;
    }

    const { itemStats, topProfitStats } = data.recursiveStats;

    let items = topProfitStats;
    items.sort(({ top: a }, { top: b }) => optSub(a.profit, b.profit).unwrapOr(Number.MIN_SAFE_INTEGER));
    items.reverse();
    items = items.filter(({ top }) => maxVelocityOf(itemStats[top.itemId], data.isHq) >= data.minVelocity);
    items = items.slice(0, data.limit);

    let index = 0;
    const rows = [];
    for (const { top, children } of items) {
        const allProfitStats = [top, ...children];
        for (const info of allProfitStats) {
            const stats = itemStats[info.itemId];
            const key = info.key.join("|");

            rows.push({
                key,
                row: {
                    _key: key,
                    index,
                    item: { itemId: info.itemId, count: info.count },
                    hasChildren: info.hasChildren,
                    numListings: selectQuality(stats.numListings, data.isHq),
                    totalNumListings: selectQuality(stats.totalNumListings, data.isHq),
                    perBiWeek: selectQuality(stats.velocityWeeks, data.isHq),
                    count: selectQuality(stats.sellCount, data.isHq),
                    sell: info.sell,
                    buy: info.buy,
                    craft: info.craft,
                    profit: info.profit,
                }
            });
        }

        index += 1;
    }

    data.tableRows = rows;
}
