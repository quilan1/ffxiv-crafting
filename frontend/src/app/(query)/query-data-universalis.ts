import { RecursiveStats } from "../(universalis)/analysis";
import { allRecursiveStatsOfAsync } from "../(universalis)/analysis-async";
import { maxVelocityOf } from "../(universalis)/statistics";
import { optSub } from "../(util)/option";
import { tryParse } from "../(util)/util";
import { ChangedState } from "./query-data";
import { QueryDataCalc } from "./query-data-calc";
import { QueryDataUi } from "./query-data-ui";
import { KeyedTableRow } from "./table";

export const recalculateUniversalis = async (
    ui: QueryDataUi, calculated: QueryDataCalc, changedStates: Set<ChangedState>
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
        const recursiveStats = await recalculateRecStatistics(ui, calculated);
        calculated = { ...calculated, recursiveStats };
    }

    calculated = recalculateTableRows(ui, calculated);

    if (changedStates.has(ChangedState.UNIVERSALIS_INFO)
        || changedStates.has(ChangedState.IS_HQ)) {
        const hiddenKeys = new Set<string>();
        for (const { key, row } of calculated.tableRows ?? []) {
            if (row.buy.unwrap_or(Number.MAX_SAFE_INTEGER) < row.craft.unwrap_or(Number.MIN_SAFE_INTEGER)) {
                hiddenKeys.add(key);
            }
        }
        calculated = { ...calculated, hiddenKeys };
    }

    return { ...calculated };
}

const recalculateRecStatistics = async (ui: QueryDataUi, calculated: QueryDataCalc): Promise<RecursiveStats | undefined> => {
    if (calculated.universalisInfo === undefined) return undefined;
    const _count = tryParse(ui.count).unwrap_or(1);
    return await allRecursiveStatsOfAsync(_count, ui.isHq, calculated.universalisInfo)
}

const recalculateTableRows = (ui: QueryDataUi, calculated: QueryDataCalc): QueryDataCalc => {
    if (calculated.universalisInfo === undefined || calculated.recursiveStats === undefined) return calculated;
    const _limit = tryParse(ui.limit).unwrap_or(100);
    const _minVelocity = tryParse(ui.minVelocity).unwrap_or(0);
    const tableRows = generateTableData(_limit, _minVelocity, calculated.recursiveStats);
    return { ...calculated, tableRows };
}

function generateTableData(limit: number, minVelocity: number, recursiveStats: RecursiveStats): KeyedTableRow[] {
    const { itemStats, topProfitStats } = recursiveStats;

    let items = topProfitStats;
    items.sort(({ top: a }, { top: b }) => optSub(a.profit, b.profit).unwrap_or(Number.MIN_SAFE_INTEGER));
    items.reverse();
    items = items.filter(({ top }) => maxVelocityOf(itemStats[top.itemId]) >= minVelocity);
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
                    perDay: stats.velocityDay.aq,
                    perWeek: stats.velocityWeek.aq,
                    perBiWeek: stats.velocityWeeks.aq,
                    count: stats.sellCount.aq,
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