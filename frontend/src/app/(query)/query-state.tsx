import { dataCenterOf } from "../(universalis)/data-center";
import { ListingStatus } from "../(universalis)/universalis-api";
import { Signal, useSignal } from "../(util)/signal";
import { defaultQueryString } from "./query";
import { QueryDataState, useQueryDataState } from "./query-data";

export interface QueryState {
    queryString: Signal<string>,
    purchaseFrom: Signal<string>,
    listingStatus: Signal<ListingStatus | undefined>,
    queryData: QueryDataState,
}

export const useQueryStateDefault = (homeworld: Signal<string>) => {
    return {
        listingStatus: useSignal<ListingStatus | undefined>(undefined),
        queryString: useSignal(defaultQueryString),
        purchaseFrom: useSignal(dataCenterOf(homeworld.value)),
        queryData: useQueryDataState(homeworld),
    }
}
