import { Dispatch, SetStateAction } from "react";

export type SimpleStateUse<V> = [V, Dispatch<SetStateAction<V>>];

export class SimpleState<V> {
    readonly state: V;
    readonly setState: SimpleStateUse<V>[1];
    constructor(def: SimpleStateUse<V>) {
        const [state, setState] = def;
        this.state = state;
        this.setState = setState;
    }

    get value(): V { return this.state };
    set value(value: V) { this.setState(value); }
}
