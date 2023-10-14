import { ReactNode, createContext, useContext } from "react";
import { QueryState, useQueryStateDefault } from "./query-state";

export const QueryContext = createContext<QueryState | null>(null);

export function QueryContextProvider({ children }: { children: ReactNode }) {
    const state: QueryState = useQueryStateDefault();
    return <QueryContext.Provider value={state}>{children}</QueryContext.Provider>
}

export function useQueryContext(): QueryState {
    const context = useContext(QueryContext);
    if (context === null) {
        throw new Error('You must use QueryContext inside of a QueryContextProvider');
    }
    return context;
}
