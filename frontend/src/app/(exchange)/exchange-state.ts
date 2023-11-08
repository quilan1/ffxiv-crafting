import { atom } from "jotai";
import { Signal, useSignal } from "../(util)/signal";
import { useHomeworld } from "../(config)/config-state";
import { ExchangeInfo, fetchExchangeInfo } from "./fetch-exchange-info";
import { dataCenterOf } from "../(universalis)/data-center";
import { ListingStatus } from "../(universalis)/universalis-api";

const isFetchingAtom = atom(false);
export const useIsFetching = () => useSignal(isFetchingAtom);

type ListingStatusAtom = ReturnType<typeof newListingAtom>;
export interface ListingStatusPair<T> { price: T, profit: T };

const newListingAtom = () => atom<ListingStatus | undefined>(undefined);
const newAtomPair = (): ListingStatusPair<ListingStatusAtom> => ({ price: newListingAtom(), profit: newListingAtom() });
const useSignalPair = (atomPair: ListingStatusPair<ListingStatusAtom>): ListingStatusPair<Signal<ListingStatus | undefined>> => {
    return { price: useSignal(atomPair.price), profit: useSignal(atomPair.profit) }
}

const listingStatusesAtoms: ListingStatusPair<ListingStatusAtom>[] = [newAtomPair(), newAtomPair(), newAtomPair()];
export const useListingStatuses = () => [useSignalPair(listingStatusesAtoms[0]), useSignalPair(listingStatusesAtoms[1]), useSignalPair(listingStatusesAtoms[2])];

const infoAtom = atom<ExchangeInfo[] | undefined>(undefined);
export const useInfo = () => useSignal(infoAtom);

export const useFetchInfo = (): (() => void) => {
    const isFetching = useIsFetching();
    const homeworld = useHomeworld();
    const listingStatuses = useListingStatuses();
    const info = useInfo();

    return () => {
        void (async () => {
            if (isFetching.value) return;
            isFetching.value = true;
            info.value = await fetchExchangeInfo(listingStatuses, homeworld.value, dataCenterOf(homeworld.value));
            isFetching.value = false;
            for (const listingPair of listingStatuses) {
                listingPair.price.value = undefined;
                listingPair.profit.value = undefined;
            }
        })();
    };
}
