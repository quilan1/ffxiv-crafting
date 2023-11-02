import { allDataCenters } from "../(universalis)/data-center";
import { Signal, useSignal } from "../(util)/signal";
import { useAppContext } from "../context";

export interface ConfigState {
    homeworld: Signal<string>,
}

export function useConfigStateDefault(): ConfigState {
    return {
        homeworld: useSignal(allDataCenters[0].world, "homeworld")
    }
}

export function useHomeworld() {
    const { configState: { homeworld } } = useAppContext();
    return homeworld;
}