import { RecursiveStats } from "@/app/(universalis)/analysis";
import { UniversalisInfo } from "@/app/(universalis)/universalis-api";
import { KeyedTableRow } from "../(table)/table";
import { useSignal } from "@/app/(util)/signal";
import { atom } from "jotai";

const checkedKeysAtom = atom(new Set<string>());
export const useCheckedKeys = () => useSignal(checkedKeysAtom);

const hiddenKeysAtom = atom(new Set<string>());
export const useHiddenKeys = () => useSignal(hiddenKeysAtom);

const universalisInfoAtom = atom<UniversalisInfo | undefined>(undefined);
export const useUniversalisInfo = () => useSignal(universalisInfoAtom);

const recursiveStatsAtom = atom<RecursiveStats | undefined>(undefined);
export const useRecursiveStats = () => useSignal(recursiveStatsAtom);

const tableRowsAtom = atom<KeyedTableRow[] | undefined>(undefined);
export const useTableRows = () => useSignal(tableRowsAtom);

export const useSetCheckKey = () => {
    const checkedKeys = useCheckedKeys();
    const setChildKeys = useSetChildKeys();
    return (key: string, isSet: boolean) => {
        checkedKeys.value = setChildKeys(key, isSet, true);
    }
}

export const useToggleHiddenKey = () => {
    const hiddenKeys = useHiddenKeys();
    return (key: string) => {
        const _hiddenKeys = new Set(hiddenKeys.value)
        if (_hiddenKeys.has(key))
            _hiddenKeys.delete(key);
        else
            _hiddenKeys.add(key);
        hiddenKeys.value = _hiddenKeys;
    };
}

export const useIsChildOfHiddenKey = () => {
    const hiddenKeys = useHiddenKeys();
    return (key: string): boolean => {
        return ![...hiddenKeys.value].every(k => !key.startsWith(`${k}|`));
    }
}

const useSetChildKeys = () => {
    const checkedKeys = useCheckedKeys();
    const tableRows = useTableRows();
    return (key: string, isSet: boolean, includeSelf: boolean): Set<string> => {
        const pred = (k: string) => (includeSelf && k == key) || k.startsWith(`${key}|`);
        if (!isSet || !tableRows.value) {
            return new Set([...checkedKeys.value].filter(k => !pred(k)));
        } else {
            const set = new Set(checkedKeys.value);
            const newKeys = tableRows.value.map(r => r.key).filter(pred);
            for (const k of newKeys) set.add(k);
            return set;
        }
    };
}
