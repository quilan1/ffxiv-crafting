import { allDataCenters, dataCenterOf } from "../(universalis)/data-center";
import { ListingStatus, UniversalisRequest } from "../(universalis)/universalis-api";
import { Signal, useSignal } from "../(util)/signal";
import { defaultQuery } from "./query-processing";
import { QuerySharedState, useQuerySharedStateDefault } from "./(shared-state)/query-shared";
import { useAppContext } from "../context";
import { MutableRefObject, useEffect, useMemo, useRef } from "react";

export interface QueryState {
    queryString: Signal<string>,
    queryDropdown: Signal<string>,
    purchaseFrom: Signal<string>,
    listingStatus: Signal<ListingStatus | undefined>,
    isFetching: Signal<boolean>,
    isCancelled: MutableRefObject<boolean>,
    queryData: QuerySharedState,
}

export const useQueryStateDefault = (homeworld: Signal<string>): QueryState => {
    return {
        listingStatus: useSignal<ListingStatus | undefined>(undefined),
        queryString: useSignal(defaultQuery.query),
        queryDropdown: useSignal(defaultQuery.label),
        purchaseFrom: useSignal(dataCenterOf(homeworld.value)),
        isFetching: useSignal(false),
        isCancelled: useRef(false),
        queryData: useQuerySharedStateDefault(homeworld),
    }
}

export function useQueryState(): QueryState {
    return useAppContext().queryState;
}

export function usePurchaseFrom(): [Signal<string>, string[]] {
    const { purchaseFrom } = useQueryState();
    const { configState: { homeworld } } = useAppContext();
    const purchaseFromOptions = useMemo(() => {
        const dataCenterInfo = allDataCenters.find(info => info.world === homeworld.value);
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        return [dataCenterInfo!.world, dataCenterInfo!.dataCenter, dataCenterInfo!.region];
    }, [homeworld]);

    useEffect(() => {
        if (!purchaseFromOptions.includes(purchaseFrom.value)) {
            purchaseFrom.value = purchaseFromOptions[1];
        }
    }, [homeworld, purchaseFrom, purchaseFromOptions]);

    return [purchaseFrom, purchaseFromOptions];
}

export function useFetchQuery() {
    const { listingStatus, queryString, purchaseFrom, isFetching, isCancelled, queryData } = useQueryState();

    return () => {
        void (async () => {
            if (!isFetching.value) {
                isFetching.value = true;
                isCancelled.current = false;
                try {
                    const universalisInfo = await new UniversalisRequest(queryString.value, purchaseFrom.value)
                        .setIsCancelled(() => isCancelled.current)
                        .setStatusFn(status => { listingStatus.value = status; })
                        .fetch();

                    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
                    if (!isCancelled.current) {
                        listingStatus.value = { status: "Calculating statistics..." };
                        await queryData.setUniversalisInfo(universalisInfo ?? undefined);
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
