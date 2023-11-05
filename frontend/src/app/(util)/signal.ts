import { Atom, useAtom } from "jotai";
import { useState } from "react";
import { useLocalStorageGet, useLocalStorageSet } from "./local-storage";

export type SimpleSetter<T> = (_: T) => void;
export type SimpleValueSetter<T> = [T, SimpleSetter<T>];

export type Signaled<T extends object> = {
    [K in keyof T]-?: Signal<T[K]>
};

export class Signal<T> {
    readonly state: T;
    readonly setState: SimpleSetter<T>;
    constructor(def: SimpleValueSetter<T>) {
        const [state, setState] = def;
        this.state = state;
        this.setState = setState;
    }

    get value(): T { return this.state };
    set value(value: T) { this.setState(value); }
}

function isAtom<T>(obj: unknown): obj is Atom<T> {
    if (!obj) return false;
    if (typeof obj !== "object") return false;
    // eslint-disable-next-line @typescript-eslint/no-base-to-string
    return obj.toString().startsWith("atom");
}

function useStateOrAtom<T>(val: T | Atom<T>): SimpleValueSetter<T> {
    return (isAtom(val))
        // eslint-disable-next-line react-hooks/rules-of-hooks
        ? useAtom(val) as SimpleValueSetter<T>
        // eslint-disable-next-line react-hooks/rules-of-hooks
        : useState(val);
}

export function useSignal<T>(val: T | Atom<T>): Signal<T> {
    return new Signal(useStateOrAtom(val));
}

export function useSignalLocalStorage<T>(val: T | Atom<T>, name: string): Signal<T> {
    const [state, setState] = useStateOrAtom(val);
    return useLocalStorage([state, setState], name);
}

function useLocalStorage<T>(stateType: SimpleValueSetter<T>, name: string): Signal<T> {
    const [state, setState] = stateType;
    const namedSetState = useLocalStorageSet(name, setState);
    useLocalStorageGet(name, setState);
    return new Signal([state, namedSetState]);
}
