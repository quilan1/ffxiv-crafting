import { tryParse } from "@/app/(util)/util";
import { ChangedState } from "./query-shared";
import { QuerySharedCalc } from "./query-shared-calc";
import { QuerySharedInputs } from "./query-shared-inputs";
import { allRecursiveStatsOfAsync } from "@/app/(universalis)/analysis-async";
import { RecursiveStats } from "@/app/(universalis)/analysis";
import { KeyedTableRow } from "../table";
import { optSub } from "@/app/(util)/option";
import { maxVelocityOf, selectQuality } from "@/app/(universalis)/statistics";

export const recalculateUniversalis = async (
    inputs: QuerySharedInputs, calculated: QuerySharedCalc, changedStates: Set<ChangedState>, homeworld: string
) => {
    if (calculated.universalisInfo === undefined) {
        return { ...calculated };
    }

    if (changedStates.has(ChangedState.UNIVERSALIS_INFO)) {
        calculated = { ...calculated, checkedKeys: new Set() };
    }

    if (changedStates.has(ChangedState.UNIVERSALIS_INFO)
        || changedStates.has(ChangedState.COUNT)
        || changedStates.has(ChangedState.IS_HQ)) {
        const recursiveStats = await recalculateRecStatistics(inputs, calculated, homeworld);
        calculated = { ...calculated, recursiveStats };
    }

    calculated = recalculateTableRows(inputs, calculated);

    if (changedStates.has(ChangedState.UNIVERSALIS_INFO)
        || changedStates.has(ChangedState.IS_HQ)) {
        const hiddenKeys = new Set<string>();
        for (const { key, row } of calculated.tableRows ?? []) {
            if (row.buy.unwrapOr(Number.MAX_SAFE_INTEGER) < row.craft.unwrapOr(Number.MIN_SAFE_INTEGER)) {
                hiddenKeys.add(key);
            }
        }
        calculated = { ...calculated, hiddenKeys };
    }

    return { ...calculated };
}

const recalculateRecStatistics = async (inputs: QuerySharedInputs, calculated: QuerySharedCalc, homeworld: string): Promise<RecursiveStats | undefined> => {
    if (calculated.universalisInfo === undefined) return undefined;
    const _count = tryParse(inputs.count).unwrapOr(1);
    return await allRecursiveStatsOfAsync(_count, inputs.isHq, calculated.universalisInfo, homeworld)
}

const recalculateTableRows = (ui: QuerySharedInputs, calculated: QuerySharedCalc): QuerySharedCalc => {
    if (calculated.universalisInfo === undefined || calculated.recursiveStats === undefined) return calculated;
    const _limit = tryParse(ui.limit).unwrapOr(100);
    const _minVelocity = tryParse(ui.minVelocity).unwrapOr(0);
    const tableRows = generateTableData(_limit, _minVelocity, ui.isHq, calculated.recursiveStats);
    return { ...calculated, tableRows };
}

function generateTableData(limit: number, minVelocity: number, isHq: boolean, recursiveStats: RecursiveStats): KeyedTableRow[] {
    const { itemStats, topProfitStats } = recursiveStats;

    let items = topProfitStats;
    items.sort(({ top: a }, { top: b }) => optSub(a.profit, b.profit).unwrapOr(Number.MIN_SAFE_INTEGER));
    items.reverse();
    items = items.filter(({ top }) => maxVelocityOf(itemStats[top.itemId], isHq) >= minVelocity);
    items = items.slice(0, limit);

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
                    numListings: selectQuality(stats.numListings, isHq),
                    totalNumListings: selectQuality(stats.totalNumListings, isHq),
                    perDay: selectQuality(stats.velocityDay, isHq),
                    perWeek: selectQuality(stats.velocityWeek, isHq),
                    perBiWeek: selectQuality(stats.velocityWeeks, isHq),
                    count: selectQuality(stats.sellCount, isHq),
                    sell: info.sell,
                    buy: info.buy,
                    craft: info.craft,
                    profit: info.profit,
                }
            });
        }

        index += 1;
    }

    return rows;
}
