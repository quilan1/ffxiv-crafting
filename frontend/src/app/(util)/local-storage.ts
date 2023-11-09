import { useEffect } from "react";
import { SimpleSetter } from "./signal";

export function useLocalStorageGet<T>(name: string, setState: SimpleSetter<T>) {
    useEffect(() => {
        try {
            const value = localStorage.getItem(name);
            if (value != undefined) setState(value as T);
        } catch { }
    }, [setState, name]);
}

export function useLocalStorageSet<T>(name: string, setState?: SimpleSetter<T>): SimpleSetter<T> {
    return (value: T) => {
        try {
            localStorage.setItem(name, value as string)
        } catch { }
        if (setState) setState(value);
    };
}
