import { useEffect } from "react";
import { SimpleSetter } from "./signal";

export function useLocalStorageGet<T>(name: string, setState: SimpleSetter<T>) {
    useEffect(() => {
        const value = localStorage.getItem(name);
        if (value != undefined) setState(value as T);
    }, [setState, name]);
}

export function useLocalStorageSet<T>(name: string, setState?: SimpleSetter<T>): SimpleSetter<T> {
    return (value: T) => {
        localStorage.setItem(name, value as string)
        if (setState) setState(value);
    };
}
