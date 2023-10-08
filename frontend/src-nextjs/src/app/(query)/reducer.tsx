import { Dispatch } from "react";
import { UniversalisInfo } from "../(universalis)/universalis_api";
import Util, { KeysMatching } from "../(universalis)/util";
import { KeyedTableRow } from "./(table)/table";
import UniversalisAnalysis from "../(universalis)/analysis";

export interface QueryState {
    query: string,
    dataCenter: string,
    count: string,
    limit: string,
    minVelocity: string,
    universalisInfo: UniversalisInfo | null,
    tableRows: KeyedTableRow[] | null,
}

export enum QueryReducerAction {
    SET_QUERY,
    SET_DATA_CENTER,
    SET_LIMIT,
    SET_COUNT,
    SET_MIN_VELOCITY,
    SET_UNIVERSALIS_INFO,
}

interface ValidDispatchType<Action, Value> { type: Action, value: Value };
type ValidDispatch =
    ValidDispatchType<QueryReducerAction.SET_QUERY, string>
    | ValidDispatchType<QueryReducerAction.SET_COUNT, string>
    | ValidDispatchType<QueryReducerAction.SET_LIMIT, string>
    | ValidDispatchType<QueryReducerAction.SET_DATA_CENTER, string>
    | ValidDispatchType<QueryReducerAction.SET_MIN_VELOCITY, string>
    | ValidDispatchType<QueryReducerAction.SET_UNIVERSALIS_INFO, UniversalisInfo>;

export function QueryReducer(state: QueryState, action: ValidDispatch): QueryState {
    let recalc;
    switch (action.type) {
        case QueryReducerAction.SET_QUERY:
            return processQuery(action.value, { ...state });
        case QueryReducerAction.SET_DATA_CENTER:
            return { ...state, dataCenter: action.value };
        case QueryReducerAction.SET_LIMIT:
            recalc = recalculateTableRows(state.count, action.value, state.minVelocity, state.universalisInfo);
            return { ...state, ...recalc };
        case QueryReducerAction.SET_COUNT:
            recalc = recalculateTableRows(action.value, state.limit, state.minVelocity, state.universalisInfo);
            return { ...state, ...recalc };
        case QueryReducerAction.SET_MIN_VELOCITY:
            recalc = recalculateTableRows(state.count, state.limit, action.value, state.universalisInfo);
            return { ...state, ...recalc };
        case QueryReducerAction.SET_UNIVERSALIS_INFO:
            const universalisInfo = action.value;
            recalc = recalculateTableRows(state.count, state.limit, state.minVelocity, universalisInfo);
            return { ...state, ...recalc };
        default:
            return state;
    }
}

const recalculateTableRows = (count: string, limit: string, minVelocity: string, universalisInfo?: UniversalisInfo | null) => {
    if (universalisInfo == null) {
        return { count, limit, minVelocity };
    }

    const analysis = new UniversalisAnalysis(universalisInfo);
    const _count = Util.tryParse(count);
    const _limit = Util.tryParse(limit);
    const _minVelocity = Util.tryParse(minVelocity);
    const tableRows = analysis.generateTableData(_count, _limit, _minVelocity);
    return { count, limit, minVelocity, universalisInfo, tableRows };
}

export class QueryDispatcher {
    private state: QueryState;
    private dispatch: Dispatch<ValidDispatch>;
    constructor(state: QueryState, dispatch: Dispatch<ValidDispatch>) {
        this.state = state;
        this.dispatch = dispatch;
    }

    get query() { return this.state.query; }
    set query(value: string) { this.dispatch({ type: QueryReducerAction.SET_QUERY, value }); }
    get dataCenter() { return this.state.dataCenter; }
    set dataCenter(value: string) { this.dispatch({ type: QueryReducerAction.SET_DATA_CENTER, value }); }
    get count() { return this.state.count; }
    set count(value: string) { this.dispatch({ type: QueryReducerAction.SET_COUNT, value }); }
    get limit() { return this.state.limit; }
    set limit(value: string) { this.dispatch({ type: QueryReducerAction.SET_LIMIT, value }); }
    get minVelocity() { return this.state.minVelocity; }
    set minVelocity(value: string) { this.dispatch({ type: QueryReducerAction.SET_MIN_VELOCITY, value }); }
    get universalisInfo() { return this.state.universalisInfo; }
    set universalisInfo(value: UniversalisInfo | null) {
        if (value !== null) this.dispatch({ type: QueryReducerAction.SET_UNIVERSALIS_INFO, value });
    }
    get tableRows() { return this.state.tableRows; }
}

export function processQuery(queryString: string, state: QueryState): QueryState {
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
