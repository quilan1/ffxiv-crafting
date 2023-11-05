import { atom } from "jotai";
import { useSignal } from "../(util)/signal";
import { useHomeworld } from "../(config)/config-state";
import { ExchangeInfo, fetchExchangeInfo } from "./fetch-exchange-info";
import { dataCenterOf } from "../(universalis)/data-center";

const isFetchingAtom = atom(false);
export const useIsFetching = () => useSignal(isFetchingAtom);

const statusesAtoms = [atom(""), atom(""), atom("")];
export const useStatuses = () => [useSignal(statusesAtoms[0]), useSignal(statusesAtoms[1]), useSignal(statusesAtoms[2])];

const infoAtom = atom<ExchangeInfo[] | undefined>(undefined);
export const useInfo = () => useSignal(infoAtom);

export const useFetchInfo = (): (() => void) => {
    const isFetching = useIsFetching();
    const homeworld = useHomeworld();
    const statuses = useStatuses();
    const info = useInfo();

    return () => {
        void (async () => {
            if (isFetching.value) return;
            isFetching.value = true;
            info.value = await fetchExchangeInfo(statuses, homeworld.value, dataCenterOf(homeworld.value));
            isFetching.value = false;
            statuses[0].value = "";
            statuses[1].value = "";
            statuses[2].value = "";
        })();
    };
}
