import { useState } from "react";
import { ListingStatusInfo } from "../(universalis)/universalis-api";
import { Signal, useSignal } from "../(util)/signal";
import { defaultDataCenter, defaultQueryString } from "./query";
import { QueryDataState, useQueryDataState } from "./query-data";

export interface QueryState {
    queryString: Signal<string>,
    dataCenter: Signal<string>,
    listingStatusInfo: Signal<ListingStatusInfo | undefined>,
    queryData: QueryDataState,
}

export const useQueryStateDefault = () => {
    return {
        listingStatusInfo: useSignal(useState<ListingStatusInfo | undefined>(undefined)),
        queryString: useSignal(useState(defaultQueryString)),
        dataCenter: useSignal(useState(defaultDataCenter)),
        queryData: useQueryDataState(),
    }
}
