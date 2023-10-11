import { useState } from "react";
import { ListingStatusInfo } from "../(universalis)/universalis_api";
import { Signal, signal } from "../(universalis)/signal";
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
        listingStatusInfo: signal(useState<ListingStatusInfo | undefined>(undefined)),
        queryString: signal(useState(defaultQueryString)),
        dataCenter: signal(useState(defaultDataCenter)),
        queryData: useQueryDataState(),
    }
}
