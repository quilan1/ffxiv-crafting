import { RecursiveStats } from "@/app/(universalis)/analysis";
import { UniversalisInfo } from "@/app/(universalis)/universalis-api";
import { KeyedTableRow } from "../table";
import { Signaled, useSignal } from "@/app/(util)/signal";

export interface QuerySharedCalc {
    checkedKeys: Set<string>,
    hiddenKeys: Set<string>,
    universalisInfo?: UniversalisInfo,
    recursiveStats?: RecursiveStats,
    tableRows?: KeyedTableRow[],
}

export function useQuerySharedCalcDefault(): Signaled<QuerySharedCalc> {
    const checkedKeys = useSignal(new Set<string>());
    const hiddenKeys = useSignal(new Set<string>());
    const universalisInfo = useSignal<UniversalisInfo | undefined>(undefined);
    const recursiveStats = useSignal<RecursiveStats | undefined>(undefined);
    const tableRows = useSignal<KeyedTableRow[] | undefined>(undefined);
    return { checkedKeys, hiddenKeys, universalisInfo, recursiveStats, tableRows };
}

export class QuerySharedCalcState {
    private _state: Signaled<QuerySharedCalc>;
    constructor(state: Signaled<QuerySharedCalc>) {
        this._state = state;
    }

    get state() { return this._state; }
    get values(): QuerySharedCalc {
        return {
            checkedKeys: this.state.checkedKeys.value,
            hiddenKeys: this.state.hiddenKeys.value,
            universalisInfo: this.state.universalisInfo.value,
            recursiveStats: this.state.recursiveStats.value,
            tableRows: this.state.tableRows.value,
        }
    }
    set values(values: QuerySharedCalc) {
        this.state.checkedKeys.value = values.checkedKeys;
        this.state.hiddenKeys.value = values.hiddenKeys;
        this.state.universalisInfo.value = values.universalisInfo;
        this.state.recursiveStats.value = values.recursiveStats;
        this.state.tableRows.value = values.tableRows;
    }

    get universalisInfo() { return this._state.universalisInfo; }
    get tableRows() { return this._state.tableRows; }
    get checkedKeys() { return this._state.checkedKeys; }
    get hiddenKeys() { return this._state.hiddenKeys; }
    get recursiveStats() { return this._state.recursiveStats; }

    setCheckKey(key: string, isSet: boolean) {
        this._state.checkedKeys.value = this.setChildKeys(this._state.checkedKeys.value, key, isSet, true);
    }

    toggleHiddenKey(key: string) {
        const hiddenKeys = new Set(this._state.hiddenKeys.value);
        if (hiddenKeys.has(key))
            hiddenKeys.delete(key);
        else
            hiddenKeys.add(key);
        this._state.hiddenKeys.value = hiddenKeys;
    }

    isChildOfHiddenKey(key: string): boolean {
        return ![...this.hiddenKeys.value].every(k => !key.startsWith(`${k}|`));
    }

    private setChildKeys(keySet: Set<string>, key: string, isSet: boolean, includeSelf: boolean): Set<string> {
        const pred = (k: string) => (includeSelf && k == key) || k.startsWith(`${key}|`);
        if (!isSet || !this._state.tableRows.value) {
            return new Set([...keySet].filter(k => !pred(k)));
        } else {
            const set = new Set(keySet);
            const newKeys = this._state.tableRows.value.map(r => r.key).filter(pred);
            for (const k of newKeys) set.add(k);
            return set;
        }
    }
}
