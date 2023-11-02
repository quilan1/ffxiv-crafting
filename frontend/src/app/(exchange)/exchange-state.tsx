import { Signal, useSignal } from "../(util)/signal";
import { ExchangeInfo } from "./exchange";

export interface ExchangeState {
    isFetching: Signal<boolean>,
    statuses: Signal<string>[],
    info: Signal<ExchangeInfo[] | undefined>;
}

export const useExchangeStateDefault = (): ExchangeState => {
    return {
        isFetching: useSignal(false),
        statuses: [useSignal(""), useSignal(""), useSignal("")],
        info: useSignal<ExchangeInfo[] | undefined>(undefined),
    };
}