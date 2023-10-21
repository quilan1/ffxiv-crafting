import { ListingStatus } from "../(universalis)/universalis-api";
import { Signal, useSignal } from "../(util)/signal";
import { defaultDataCenter, defaultQueryString } from "./query";
import { QueryDataState, useQueryDataState } from "./query-data";

export interface QueryState {
    queryString: Signal<string>,
    dataCenter: Signal<string>,
    listingStatus: Signal<ListingStatus | undefined>,
    queryData: QueryDataState,
}

export const useQueryStateDefault = () => {
    return {
        listingStatus: useSignal<ListingStatus | undefined>(undefined),
        queryString: useSignal(defaultQueryString),
        dataCenter: useSignal(defaultDataCenter),
        queryData: useQueryDataState(),
    }
}
