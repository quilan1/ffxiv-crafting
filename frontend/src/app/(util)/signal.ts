import { Dispatch, SetStateAction, useEffect, useState } from "react";

export type SimpleStateUse<T> = [T, Dispatch<SetStateAction<T>>];

export type Signaled<T extends object> = {
    [K in keyof T]-?: Signal<T[K]>
};

export class Signal<T> {
    readonly state: T;
    readonly setState: SimpleStateUse<T>[1];
    readonly name?: string;
    constructor(def: SimpleStateUse<T>, name?: string) {
        const [state, setState] = def;
        this.state = state;
        this.setState = setState;
        this.name = name;
    }

    get value(): T { return this.state };
    set value(value: T) {
        if (this.name) localStorage.setItem(this.name, value as string);
        this.setState(value);
    }
}

export function useSignal<T>(val: T, name?: string): Signal<T> {
    const [state, setState] = useState(val);

    useEffect(() => {
        if (name !== undefined) {
            setState((localStorage.getItem(name) ?? val) as T);
        }
    }, [name, val]);

    return new Signal([state, setState], name);
}
