import { useState } from "react";
import { RecursiveStats, allRecursiveStatsOfAsync } from "../(universalis)/analysis";
import { optSub } from "../(universalis)/option";
import { SimpleStateUse } from "../(universalis)/signal";
import { maxVelocityOf } from "../(universalis)/statistics";
import { UniversalisInfo } from "../(universalis)/universalis_api";
import Util from "../(universalis)/util";
import { KeyedTableRow } from "./table";

export interface QueryData {
    count: string,
    limit: string,
    minVelocity: string,
    isHq: boolean,
    checkedKeys: Set<string>,
    hiddenKeys: Set<string>,
    universalisInfo?: UniversalisInfo,
    recursiveStats?: RecursiveStats,
    tableRows?: KeyedTableRow[],
}

export function useQueryDataState() {
    return new QueryDataState(useState<QueryData>({
        count: '100',
        limit: '',
        minVelocity: '',
        isHq: false,
        checkedKeys: new Set<string>(),
        hiddenKeys: new Set<string>(),
    }));
}

enum ChangedState {
    COUNT,
    LIMIT,
    MIN_VELOCITY,
    IS_HQ,
    UNIVERSALIS_INFO,
}

export class QueryDataState {
    private _state: QueryData;
    private setState: SimpleStateUse<QueryData>[1];
    constructor(signal: SimpleStateUse<QueryData>) {
        const [state, setState] = signal;
        this._state = state;
        this.setState = setState;
    }

    get state() { return this._state; }
    set state(value: QueryData) { this.setState({ ...value }); }

    get count() { return this._state.count; }
    set count(count: string) { void this.recalculateUniversalis({ ...this._state, count }, ChangedState.COUNT); }
    get limit() { return this._state.limit; }
    set limit(limit: string) { void this.recalculateUniversalis({ ...this._state, limit }, ChangedState.LIMIT); }
    get minVelocity() { return this._state.minVelocity; }
    set minVelocity(minVelocity: string) { void this.recalculateUniversalis({ ...this._state, minVelocity }, ChangedState.MIN_VELOCITY); }
    get isHq() { return this._state.isHq; }
    set isHq(value: boolean) { void this.recalculateUniversalis({ ...this._state, isHq: value }, ChangedState.IS_HQ); }
    get universalisInfo() { return this._state.universalisInfo; }
    async setUniversalisInfo(value: UniversalisInfo | undefined): Promise<void> {
        await this.recalculateUniversalis({ ...this._state, universalisInfo: value }, ChangedState.UNIVERSALIS_INFO);
    }
    get tableRows() { return this._state.tableRows; }
    get checkedKeys() { return this._state.checkedKeys; }
    setCheckKey(key: string, isSet: boolean) {
        const checkedKeys = this.setChildKeys(this._state.checkedKeys, key, isSet, true);
        this.setState({ ...this._state, checkedKeys });
    }
    get hiddenKeys() { return this._state.hiddenKeys; }
    toggleHiddenKey(key: string) {
        const hiddenKeys = new Set(this._state.hiddenKeys);
        if (hiddenKeys.has(key))
            hiddenKeys.delete(key);
        else
            hiddenKeys.add(key);
        this.setState({ ...this._state, hiddenKeys });
    }
    isChildOfHiddenKey(key: string): boolean {
        return ![...this.hiddenKeys].every(k => !key.startsWith(`${k}|`));
    }

    private async recalculateUniversalis(state: QueryData, changedState: ChangedState) {
        if (state.universalisInfo === undefined) {
            this.setState({ ...state });
            return;
        }

        switch (changedState) {
            case ChangedState.UNIVERSALIS_INFO:
                state = { ...state, checkedKeys: new Set() };
            case ChangedState.COUNT:
            case ChangedState.IS_HQ:
                const recursiveStats = await this.recalculateRecStatistics(state);
                state = { ...state, recursiveStats };
            default:
                state = this.recalculateTableRows(state);
        }

        switch (changedState) {
            case ChangedState.UNIVERSALIS_INFO:
            case ChangedState.IS_HQ:
                const hiddenKeys = new Set<string>();
                for (const { key, row } of state.tableRows ?? []) {
                    if (row.buy.unwrap_or(Number.MAX_SAFE_INTEGER) < row.craft.unwrap_or(Number.MIN_SAFE_INTEGER)) {
                        hiddenKeys.add(key);
                    }
                }
                state = { ...state, hiddenKeys };
            default:
        }

        this.setState({ ...state });
    }

    private async recalculateRecStatistics(state: QueryData): Promise<RecursiveStats | undefined> {
        if (state.universalisInfo === undefined) return undefined;
        const _count = Util.tryParse(state.count).unwrap_or(1);
        return await allRecursiveStatsOfAsync(_count, state.isHq, state.universalisInfo)
    }

    private recalculateTableRows(state: QueryData): QueryData {
        if (state.universalisInfo === undefined || state.recursiveStats === undefined) return state;
        const _limit = Util.tryParse(state.limit).unwrap_or(100);
        const _minVelocity = Util.tryParse(state.minVelocity).unwrap_or(0);
        const tableRows = generateTableData(_limit, _minVelocity, state.universalisInfo, state.recursiveStats);
        return { ...state, tableRows };
    }

    private setChildKeys(keySet: Set<string>, key: string, isSet: boolean, includeSelf: boolean): Set<string> {
        const pred = (k: string) => (includeSelf && k == key) || k.startsWith(`${key}|`);
        if (!isSet || !this._state.tableRows) {
            return new Set([...keySet].filter(k => !pred(k)));
        } else {
            const set = new Set(keySet);
            const newKeys = this._state.tableRows.map(r => r.key).filter(pred);
            for (const k of newKeys) set.add(k);
            return set;
        }
    }
}

function generateTableData(
    limit: number, minVelocity: number, universalisInfo: UniversalisInfo, recursiveStats: RecursiveStats
): KeyedTableRow[] {
    const itemInfo = universalisInfo.itemInfo;
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
            const quantity = info.count > 1 ? `${info.count}x ` : '';
            const name = itemInfo[info.itemId].name;
            const key = info.key.join("|");

            rows.push({
                key,
                row: {
                    _key: key,
                    index,
                    itemId: info.itemId,
                    hasChildren: info.hasChildren,
                    name: `${quantity}${name}`,
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
