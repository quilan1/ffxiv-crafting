import { allDataCenters, dataCenterOf, defaultDataCenter } from "../(universalis)/data-center";
import { ListingStatus, UniversalisRequest } from "../(universalis)/universalis-api";
import { Signal, useSignal } from "../(util)/signal";
import { defaultQuery } from "./query-processing";
import { useQueryShared, updateUniversalisInfo } from "./(shared-state)/query-shared";
import { useEffect, useMemo } from "react";
import { useHomeworld } from "../(config)/config-state";
import { atom } from "jotai";
import { useCheckedKeys, useHiddenKeys, useIsChildOfHiddenKey, useTableRows, useUniversalisInfo } from "./(shared-state)/query-shared-calc";
import { FailureInfo, PurchaseWorldInfo } from "./(purchase)/purchase";
import { calculatePurchases } from "../(universalis)/purchases";
import { Ingredient } from "../(universalis)/items";
import { entriesOf } from "../(util)/util";

export interface PurchaseOption {
    label: string,
    value: string,
}

const queryStringAtom = atom(defaultQuery.query);
export const useQueryString = () => useSignal(queryStringAtom);

const queryDropdownAtom = atom(defaultQuery.label);
export const useQueryDropdown = () => useSignal(queryDropdownAtom);

const listingStatusAtom = atom<ListingStatus | undefined>(undefined);
export const useListingStatus = () => useSignal(listingStatusAtom);

const isFetchingAtom = atom(false);
export const useIsFetching = () => useSignal(isFetchingAtom);

const purchaseFromAtom = atom(dataCenterOf(defaultDataCenter.world));
export const usePurchaseFrom = (): Signal<string> => useSignal(purchaseFromAtom);

const isCancelledAtom = atom(() => ({ current: false }));
const useIsCancelled = () => useSignal(isCancelledAtom).value;

const isQueryMinimizedAtom = atom(false);
export const useIsQueryMinimized = () => useSignal(isQueryMinimizedAtom);

const isTableMinimizedAtom = atom(false);
export const useIsTableMinimized = () => useSignal(isTableMinimizedAtom);

export function usePurchaseFromData(): [Signal<string>, PurchaseOption[]] {
    const purchaseFrom = usePurchaseFrom();
    const homeworld = useHomeworld();
    const purchaseFromOptions = useMemo(() => {
        const dataCenter = dataCenterOf(homeworld.value);
        const dataCenterInfo = allDataCenters.filter(info => info.dataCenter === dataCenter);
        const results = [
            { value: dataCenterInfo[0].region, label: 'Cross-DC' },
            { value: dataCenterInfo[0].dataCenter, label: 'Cross-World' },
        ];

        for (const info of dataCenterInfo) {
            results.push({ value: info.world, label: info.world });
        }

        return results;
    }, [homeworld]);

    useEffect(() => {
        if (!purchaseFromOptions.map(options => options.value).includes(purchaseFrom.value)) {
            purchaseFrom.value = purchaseFromOptions[1].value;
        }
    }, [homeworld, purchaseFrom, purchaseFromOptions]);

    return [purchaseFrom, purchaseFromOptions];
}

export function useFetchQuery() {
    const purchaseFrom = usePurchaseFrom();
    const listingStatus = useListingStatus();
    const isFetching = useIsFetching();
    const homeworld = useHomeworld();
    const queryString = useQueryString();
    const isCancelled = useIsCancelled();
    const [data, setData] = useQueryShared();

    return () => {
        void (async () => {
            if (!isFetching.value) {
                isFetching.value = true;
                isCancelled.current = false;
                try {
                    const universalisInfo = await new UniversalisRequest(queryString.value, purchaseFrom.value, homeworld.value)
                        .setIsCancelled(() => isCancelled.current)
                        .setStatusFn(status => { listingStatus.value = status; })
                        .fetch();

                    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                    if (!isCancelled.current && universalisInfo) {
                        listingStatus.value = { status: "Calculating statistics..." };
                        setData(await updateUniversalisInfo(data, universalisInfo));
                    }
                } finally {
                    listingStatus.value = undefined;
                    isFetching.value = false;
                    isCancelled.current = false;
                }
            } else {
                isCancelled.current = true;
            }
        })()
    };
};

export const usePurchaseInfo = () => {
    const items = useCollectCheckedItems();
    const universalisInfo = useUniversalisInfo();
    const homeworld = useHomeworld();

    return useMemo(() => {
        const itemInfo = universalisInfo.value?.itemInfo ?? {};

        // build the world info
        const failures: FailureInfo[] = [];
        const purchases: PurchaseWorldInfo = {};
        for (const { itemId, count } of items) {
            // Calculate listings
            const usedListings = calculatePurchases(itemInfo[itemId].listings, count);
            if (usedListings == undefined) {
                failures.push({
                    itemName: itemInfo[itemId].name,
                    count,
                });
                continue;
            }

            /* eslint-disable @typescript-eslint/no-unnecessary-condition */
            for (const listing of usedListings) {
                const world = listing.world ?? homeworld.value;
                const usedCount = listing.count;
                const dataCenter = dataCenterOf(world);
                purchases[dataCenter] ??= {};
                purchases[dataCenter][world] ??= [];
                purchases[dataCenter][world].push({
                    itemName: itemInfo[itemId].name,
                    name: listing.name ?? "",
                    price: Math.floor(listing.price / 1.05),
                    count: usedCount,
                });
            }
            /* eslint-enable @typescript-eslint/no-unnecessary-condition */
        }

        return {
            failures,
            purchases
        };
    }, [universalisInfo, homeworld, items]);
}

const useCollectCheckedItems = (): Ingredient[] => {
    const tableRows = useTableRows();
    const checkedKeys = useCheckedKeys();
    const hiddenKeys = useHiddenKeys();
    const isChildOfHiddenKey = useIsChildOfHiddenKey();

    return useMemo(() => {
        const checkedItems = tableRows.value
            ?.filter(({ row }) => row.item.itemId > 19)
            ?.filter(({ row, key }) => !row.hasChildren || hiddenKeys.value.has(key))
            ?.filter(({ key }) => !isChildOfHiddenKey(key))
            ?.filter(({ key }) => checkedKeys.value.has(key))
            ?.map(({ row }) => row.item)
            ?? [];

        const items: Record<number, number | undefined> = {};
        for (const item of checkedItems) {
            items[item.itemId] = (items[item.itemId] ?? 0) + item.count;
        }

        return entriesOf(items as Record<number, number>)
            .map(([key, val]) => ({ itemId: key, count: val }));
    }, [checkedKeys, hiddenKeys, isChildOfHiddenKey, tableRows]);
}
