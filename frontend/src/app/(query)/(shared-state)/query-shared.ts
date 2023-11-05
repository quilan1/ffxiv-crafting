import { UniversalisInfo } from "@/app/(universalis)/universalis-api";
import { useDeferredFn } from "@/app/(util)/deferred-fn";
import { ChangedState, recalculateUniversalis } from "./recalculate-universalis";
import { useCheckedKeys, useHiddenKeys, useRecursiveStats, useTableRows, useUniversalisInfo } from "./query-shared-calc";
import { useCount, useIsHq, useLimit, useMinVelocity } from "./query-shared-inputs";
import { useHomeworld } from "@/app/(config)/config-state";
import { tryParse } from "@/app/(util)/util";
import { RecursiveStats } from "@/app/(universalis)/analysis";
import { KeyedTableRow } from "../table";
import { atom } from "jotai";
import { useSignal } from "@/app/(util)/signal";

interface ChangedValues {
    count?: string,
    limit?: string,
    minVelocity?: string,
    isHq?: boolean,
    universalisInfo?: UniversalisInfo,
}

export interface QueryShared {
    count: number,
    limit: number,
    minVelocity: number,
    isHq: boolean,
    checkedKeys: Set<string>,
    hiddenKeys: Set<string>,
    homeworld: string,
    universalisInfo?: UniversalisInfo,
    recursiveStats?: RecursiveStats,
    tableRows?: KeyedTableRow[],
}

export const useQueryShared = (): [QueryShared, (_: QueryShared) => void] => {
    const count = useCount();
    const limit = useLimit();
    const minVelocity = useMinVelocity();
    const isHq = useIsHq();
    const universalisInfo = useUniversalisInfo();
    const checkedKeys = useCheckedKeys();
    const hiddenKeys = useHiddenKeys();
    const recursiveStats = useRecursiveStats();
    const tableRows = useTableRows();
    const homeworld = useHomeworld();

    const data = {
        count: tryParse(count.value).unwrapOr(1),
        limit: tryParse(limit.value).unwrapOr(100),
        minVelocity: tryParse(minVelocity.value).unwrapOr(0),
        isHq: isHq.value,
        universalisInfo: universalisInfo.value,
        checkedKeys: checkedKeys.value,
        hiddenKeys: hiddenKeys.value,
        recursiveStats: recursiveStats.value,
        tableRows: tableRows.value,
        homeworld: homeworld.value,
    }

    const set = (data: QueryShared) => {
        universalisInfo.value = data.universalisInfo;
        checkedKeys.value = data.checkedKeys;
        hiddenKeys.value = data.hiddenKeys;
        recursiveStats.value = data.recursiveStats;
        tableRows.value = data.tableRows;
    }

    return [data, set];
}

export const updateUniversalisInfo = async (data: QueryShared, universalisInfo: UniversalisInfo): Promise<QueryShared> => {
    return await recalculateDeferred(data, { universalisInfo });
}

const allChangedValues = atom<{ current: ChangedValues }>(() => ({ current: {} }));
const useAllChangedValues = () => useSignal(allChangedValues).value;

export const useUpdateUniversalis = () => {
    const deferredFn = useDeferredFn(20);
    const allChangedValues = useAllChangedValues();
    const [data, setData] = useQueryShared();

    return (changedValues: ChangedValues) => {
        allChangedValues.current.count = changedValues.count ?? allChangedValues.current.count;
        allChangedValues.current.limit = changedValues.limit ?? allChangedValues.current.limit;
        allChangedValues.current.minVelocity = changedValues.minVelocity ?? allChangedValues.current.minVelocity;
        allChangedValues.current.isHq = changedValues.isHq ?? allChangedValues.current.isHq;
        deferredFn(async () => {
            const values = { ...allChangedValues.current };
            allChangedValues.current = {};
            setData(await recalculateDeferred(data, values));
        });
    };
}

const recalculateDeferred = async (data: QueryShared, changedValues: ChangedValues): Promise<QueryShared> => {
    const changedStates = new Set<ChangedState>();
    if (changedValues.count !== undefined) {
        changedStates.add(ChangedState.COUNT);
        data.count = tryParse(changedValues.count).unwrapOr(1);
    }
    if (changedValues.limit !== undefined) {
        changedStates.add(ChangedState.LIMIT);
        data.limit = tryParse(changedValues.limit).unwrapOr(100);
    }
    if (changedValues.minVelocity !== undefined) {
        changedStates.add(ChangedState.MIN_VELOCITY);
        data.minVelocity = tryParse(changedValues.minVelocity).unwrapOr(0);
    }

    if (changedValues.isHq !== undefined) changedStates.add(ChangedState.IS_HQ);
    if (changedValues.universalisInfo !== undefined) changedStates.add(ChangedState.UNIVERSALIS_INFO);
    data.isHq = changedValues.isHq ?? data.isHq;
    data.universalisInfo = changedValues.universalisInfo ?? data.universalisInfo;

    await recalculateUniversalis(data, changedStates);
    return data;
}
