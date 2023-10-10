import { Dispatch } from "react";
import { UniversalisInfo } from "../(universalis)/universalis_api";
import Util, { KeysMatching } from "../(universalis)/util";
import { KeyedTableRow } from "./(table)/table";
import { Statistics } from "../(universalis)/statistics";
import { RecursiveStats, allRecursiveStatsOf } from "../(universalis)/analysis";
import { defaultDataCenter, preparedQueries } from "./query";
import { optSub } from "../(universalis)/option";

export interface QueryState {
    query: string,
    dataCenter: string,
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

export enum QueryReducerAction {
    SET_QUERY,
    SET_DATA_CENTER,
    SET_CHECKED_KEYS,
    SET_HIDDEN_KEYS,
    UPDATE_STATE,
}

interface ValidDispatchType<Action, Value> { type: Action, value: Value };
type ValidDispatch =
    ValidDispatchType<QueryReducerAction.SET_QUERY, string>
    | ValidDispatchType<QueryReducerAction.SET_DATA_CENTER, string>
    | ValidDispatchType<QueryReducerAction.SET_CHECKED_KEYS, Set<string>>
    | ValidDispatchType<QueryReducerAction.SET_HIDDEN_KEYS, Set<string>>
    | ValidDispatchType<QueryReducerAction.UPDATE_STATE, QueryState>

export function QueryReducer(state: QueryState, action: ValidDispatch): QueryState {
    switch (action.type) {
        case QueryReducerAction.SET_QUERY:
            return { ...state, query: action.value };
        case QueryReducerAction.SET_DATA_CENTER:
            return { ...state, dataCenter: action.value };
        case QueryReducerAction.SET_CHECKED_KEYS:
            return { ...state, checkedKeys: action.value };
        case QueryReducerAction.SET_HIDDEN_KEYS:
            return { ...state, hiddenKeys: action.value };
        case QueryReducerAction.UPDATE_STATE:
            return { ...state, ...action.value };
        default:
            const _check: never = action;
            return state;
    }
}

enum ChangedState {
    COUNT,
    LIMIT,
    MIN_VELOCITY,
    IS_HQ,
    UNIVERSALIS_INFO,
}

export class QueryDispatcher {
    private state: QueryState;
    private dispatch: Dispatch<ValidDispatch>;
    constructor(state: QueryState, dispatch: Dispatch<ValidDispatch>) {
        this.state = state;
        this.dispatch = dispatch;
    }

    get query() { return this.state.query; }
    set query(value: string) { this.dispatch({ type: QueryReducerAction.SET_QUERY, value }) }
    setQueryWithProcessing(value: string) {
        const state = processQuery(value, { ...this.state });
        this.dispatch({ type: QueryReducerAction.UPDATE_STATE, value: state });
    }
    get dataCenter() { return this.state.dataCenter; }
    set dataCenter(value: string) { this.dispatch({ type: QueryReducerAction.SET_DATA_CENTER, value }); }
    get count() { return this.state.count; }
    set count(value: string) {
        const state = this.recalculateUniversalis({ ...this.state, count: value }, ChangedState.COUNT);
        this.dispatch({ type: QueryReducerAction.UPDATE_STATE, value: state });
    }
    get limit() { return this.state.limit; }
    set limit(value: string) {
        const state = this.recalculateUniversalis({ ...this.state, limit: value }, ChangedState.LIMIT);
        this.dispatch({ type: QueryReducerAction.UPDATE_STATE, value: state });
    }
    get minVelocity() { return this.state.minVelocity; }
    set minVelocity(value: string) {
        const state = this.recalculateUniversalis({ ...this.state, minVelocity: value }, ChangedState.MIN_VELOCITY);
        this.dispatch({ type: QueryReducerAction.UPDATE_STATE, value: state });
    }
    get isHq() { return this.state.isHq; }
    set isHq(value: boolean) {
        const state = this.recalculateUniversalis({ ...this.state, isHq: value }, ChangedState.IS_HQ);
        this.dispatch({ type: QueryReducerAction.UPDATE_STATE, value: state });
    }
    get universalisInfo() { return this.state.universalisInfo; }
    set universalisInfo(value: UniversalisInfo | undefined) {
        const state = this.recalculateUniversalis({ ...this.state, universalisInfo: value }, ChangedState.UNIVERSALIS_INFO);
        this.dispatch({ type: QueryReducerAction.UPDATE_STATE, value: state });
    }
    get tableRows() { return this.state.tableRows; }
    get checkedKeys() { return this.state.checkedKeys; }
    setCheckKey(key: string, isSet: boolean) {
        const newKeys = this.setChildKeys(this.state.checkedKeys, key, isSet, true);
        this.dispatch({ type: QueryReducerAction.SET_CHECKED_KEYS, value: newKeys });
    }
    get hiddenKeys() { return this.state.hiddenKeys; }
    toggleHiddenKey(key: string) {
        const newKeys = new Set(this.state.hiddenKeys);
        if (newKeys.has(key))
            newKeys.delete(key);
        else
            newKeys.add(key);
        this.dispatch({ type: QueryReducerAction.SET_HIDDEN_KEYS, value: newKeys });
    }
    isChildOfHiddenKey(key: string): boolean {
        return ![...this.hiddenKeys].every(k => !key.startsWith(`${k}|`));
    }

    private recalculateUniversalis(state: QueryState, changedState: ChangedState) {
        if (state.universalisInfo === undefined)
            return state;

        switch (changedState) {
            case ChangedState.UNIVERSALIS_INFO:
                state = { ...state, checkedKeys: new Set() };
            case ChangedState.COUNT:
            case ChangedState.IS_HQ:
                state = this.recalculateRecStatistics(state);
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

        return state;
    }

    private recalculateRecStatistics(state: QueryState): QueryState {
        if (state.universalisInfo === undefined) return state;
        const _count = Util.tryParse(state.count).unwrap_or(1);
        return { ...state, recursiveStats: allRecursiveStatsOf(_count, state.isHq, state.universalisInfo) };
    }

    private recalculateTableRows(state: QueryState): QueryState {
        if (state.universalisInfo === undefined || state.recursiveStats === undefined) return state;
        const _limit = Util.tryParse(state.limit).unwrap_or(100);
        const _minVelocity = Util.tryParse(state.minVelocity).unwrap_or(0);
        const tableRows = generateTableData(_limit, _minVelocity, state.universalisInfo, state.recursiveStats);
        return { ...state, tableRows };
    }

    private setChildKeys(keySet: Set<string>, key: string, isSet: boolean, includeSelf: boolean): Set<string> {
        const pred = (k: string) => (includeSelf && k == key) || k.startsWith(`${key}|`);
        if (!isSet || !this.state.tableRows) {
            return new Set([...keySet].filter(k => !pred(k)));
        } else {
            const set = new Set(keySet);
            const newKeys = this.state.tableRows.map(r => r.key).filter(pred);
            for (const k of newKeys) set.add(k);
            return set;
        }
    }
}

function generateTableData(
    limit: number, minVelocity: number, universalisInfo: UniversalisInfo, recursiveState: RecursiveStats
): KeyedTableRow[] {
    const itemInfo = universalisInfo.itemInfo;
    const { itemStats, topProfitStats } = recursiveState;

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

const maxVelocityOf = (stats: Statistics) => {
    const arr = [
        stats.velocityDay.aq.unwrap_or(0),
        stats.velocityWeek.aq.unwrap_or(0),
        stats.velocityWeeks.aq.unwrap_or(0)
    ].filter(v => v > 0);

    if (arr.length == 0) return 0;
    return arr.reduce((a, b) => Math.max(a, b));
}

export function defaultQueryState() {
    const defaultQuery = preparedQueries[0].value;
    const defaultState = {
        query: defaultQuery,
        dataCenter: defaultDataCenter,
        count: '',
        limit: '',
        minVelocity: '',
        isHq: false,
        checkedKeys: new Set<string>(),
        hiddenKeys: new Set<string>(),
    };
    return processQuery(defaultState.query, defaultState);
}

function processQuery(queryString: string, state: QueryState): QueryState {
    const setAndStrip = (variable: KeysMatching<QueryState, string>, regex: RegExp) => {
        const match = queryString.match(regex);
        if (match) {
            state[variable] = match[1];
            queryString = queryString.replaceAll(new RegExp(regex, 'g'), '');
        }
    }

    state.count = '';
    state.limit = '';
    state.minVelocity = '';
    setAndStrip('count', /:count ([0-9]*)\s*/);
    setAndStrip('limit', /:limit ([0-9]*)\s*/);
    setAndStrip('minVelocity', /:min_velocity ([0-9.]*)\s*/);
    while (queryString.match(/, ,/)) {
        queryString = queryString.replace(/, ,/, ',');
    }
    queryString = queryString.replace(/^,/, '');
    queryString = queryString.replace(/,$/, '');
    queryString = queryString.trim();
    state.query = queryString;
    return state;
}
