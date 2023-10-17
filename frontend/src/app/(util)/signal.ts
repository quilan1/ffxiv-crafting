import { Dispatch, SetStateAction } from "react";

export type SimpleStateUse<T> = [T, Dispatch<SetStateAction<T>>];

export class Signal<T> {
    readonly state: T;
    readonly setState: SimpleStateUse<T>[1];
    constructor(def: SimpleStateUse<T>) {
        const [state, setState] = def;
        this.state = state;
        this.setState = setState;
    }

    get value(): T { return this.state };
    set value(value: T) { this.setState(value); }
}

export function useSignal<T>(val: SimpleStateUse<T>): Signal<T> {
    return new Signal(val);
}
