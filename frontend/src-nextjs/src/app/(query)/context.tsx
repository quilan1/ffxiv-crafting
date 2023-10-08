import { ReactNode, createContext, useContext, useReducer } from "react";
import { QueryReducer, QueryDispatcher, defaultQueryState } from "./reducer";

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
