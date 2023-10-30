import { ReactNode, createContext, useContext } from "react";
import { ExchangeState, useExchangeStateDefault } from "./(exchange)/exchange-state";
import { QueryState, useQueryStateDefault } from "./(query)/query-state";
import { ConfigState, useConfigStateDefault } from "./(config)/config-state";

export interface AppState {
    queryState: QueryState,
    exchangeState: ExchangeState,
    configState: ConfigState,
}

export const useAppStateDefault = () => {
    return {
        queryState: useQueryStateDefault(),
        exchangeState: useExchangeStateDefault(),
        configState: useConfigStateDefault(),
    }
}

export const AppContext = createContext<AppState | null>(null);

export function AppContextProvider({ children }: { children: ReactNode }) {
    const state: AppState = useAppStateDefault();
    return <AppContext.Provider value={state}> {children} </AppContext.Provider>
}

export function useAppContext(): AppState {
    const context = useContext(AppContext);
    if (context === null) {
        throw new Error('You must use AppContext inside of a AppContextProvider');
    }
    return context;
}
