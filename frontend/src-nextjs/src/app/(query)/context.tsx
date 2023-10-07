import { ReactNode, createContext, useContext, useReducer } from "react";
import { UniversalisInfo } from "../(universalis)/universalis_api";
import { QueryReducer, QueryDispatcher } from "./reducer";
import { defaultQueryState } from "./query";

export interface QueryState {
    query: string,
    dataCenter: string,
    count: string,
    limit: string,
    minVelocity: string,
    universalisInfo?: UniversalisInfo,
}

export const QueryContext = createContext<QueryDispatcher | null>(null);

export default function QueryContextProvider({ children }: { children: ReactNode }) {
    const dispatcher = new QueryDispatcher(...useReducer(QueryReducer, defaultQueryState()));
    return <QueryContext.Provider value={dispatcher}>{children}</QueryContext.Provider>
}

export function useQueryContext(): QueryDispatcher {
    const context = useContext(QueryContext);
    if (context === null) {
        throw new Error('You must use QueryContext inside of a QueryContextProvider');
    }
    return context;
}
