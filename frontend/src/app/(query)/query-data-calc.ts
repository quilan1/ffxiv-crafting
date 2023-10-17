import { RecursiveStats } from "../(universalis)/analysis";
import { UniversalisInfo } from "../(universalis)/universalis-api";
import { SimpleStateUse } from "../(util)/signal";
import { KeyedTableRow } from "./table";

export interface QueryDataCalc {
    checkedKeys: Set<string>,
    hiddenKeys: Set<string>,
    universalisInfo?: UniversalisInfo,
    recursiveStats?: RecursiveStats,
    tableRows?: KeyedTableRow[],
}

export class QueryDataCalcState {
    private _state: QueryDataCalc;
    private _setState: SimpleStateUse<QueryDataCalc>[1];
    constructor(signal: SimpleStateUse<QueryDataCalc>) {
        const [state, setState] = signal;
        this._state = state;
        this._setState = setState;
    }

    get state() { return this._state; }
    set state(state: QueryDataCalc) { this._setState(state) }

    get universalisInfo() { return this._state.universalisInfo; }
    set universalisInfo(universalisInfo: UniversalisInfo | undefined) { this._setState({ ...this._state, universalisInfo }); }
    get tableRows() { return this._state.tableRows; }
    get checkedKeys() { return this._state.checkedKeys; }
    setCheckKey(key: string, isSet: boolean) {
        const checkedKeys = this.setChildKeys(this._state.checkedKeys, key, isSet, true);
        this._setState({ ...this._state, checkedKeys });
    }
    get hiddenKeys() { return this._state.hiddenKeys; }
    toggleHiddenKey(key: string) {
        const hiddenKeys = new Set(this._state.hiddenKeys);
        if (hiddenKeys.has(key))
            hiddenKeys.delete(key);
        else
            hiddenKeys.add(key);
        this._setState({ ...this._state, hiddenKeys });
    }
    isChildOfHiddenKey(key: string): boolean {
        return ![...this.hiddenKeys].every(k => !key.startsWith(`${k}|`));
    }
    get recursiveStats() { return this._state.recursiveStats; }

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

