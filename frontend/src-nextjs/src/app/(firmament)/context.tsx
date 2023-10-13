import { ReactNode, createContext, useContext } from "react";
import { FirmamentState, useFirmamentStateDefault } from "./firmament";

export const FirmamentContext = createContext<FirmamentState | null>(null);

export function FirmamentContextProvider({ children }: { children: ReactNode }) {
    const state: FirmamentState = useFirmamentStateDefault();
    return <FirmamentContext.Provider value={state}>{children}</FirmamentContext.Provider>
}

export function useFirmamentContext(): FirmamentState {
    const context = useContext(FirmamentContext);
    if (context === null) {
        throw new Error('You must use FirmamentContext inside of a FirmamentContextProvider');
    }
    return context;
}
