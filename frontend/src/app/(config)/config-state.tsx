import { atom } from "jotai";
import { defaultDataCenter } from "../(universalis)/data-center";
import { Signal, useSignalLocalStorage } from "../(util)/signal";

export interface ConfigState {
    homeworld: Signal<string>,
}

const homeworldAtom = atom(defaultDataCenter.world);

export function useHomeworld() {
    const homeworld = useSignalLocalStorage(homeworldAtom, "homeworld");
    return homeworld;
}