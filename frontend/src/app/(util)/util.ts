import { None, OptionType, Some } from "./option";

export type Mutable<T> = {
    -readonly [k in keyof T]: T[k];
};

export type KeysMatching<T extends object, V> = {
    [K in keyof T]-?: T[K] extends V ? K : never
}[keyof T];

export function keysOf<T extends Record<K, unknown>, K extends string | number | symbol>(o: T): (keyof T)[] {
    return Object.keys(o) as (keyof T)[];
};

export function entriesOf<T extends Record<K, unknown>, K extends string | number | symbol>(o: T): [keyof T, T[keyof T]][] {
    return Object.entries(o) as [keyof T, T[keyof T]][];
};

export function tryParse(s: string): OptionType<number> {
    const v = parseInt(s);
    return Number.isNaN(v) ? None() : Some(v);
}

export function cloneDeep<T>(obj: T): T {
    if (typeof obj === 'string' || typeof obj === 'number' || obj === null || obj === undefined) {
        return obj;
    } else if (Array.isArray(obj)) {
        return obj.map(value => cloneDeep(value as unknown)) as T;
    } else if (typeof obj === 'object') {
        const ret = {} as T;
        for (const [key, value] of Object.entries(obj as object)) {
            // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
            ret[key as keyof T] = cloneDeep(value);
        }
        return ret;
    } else {
        return obj;
    }
}

export function equalsDeep<T>(a: T, b: T): boolean {
    if (typeof a !== typeof b) {
        return false;
    }

    if (typeof a === 'string' || typeof a === 'number' || a === null || a === undefined ||
        typeof b === 'string' || typeof b === 'number' || b === null || b === undefined) {
        return a === b;
    } else if (Array.isArray(a)) {
        if (!Array.isArray(b)) {
            return false;
        }
        return a.length === b.length && a.every((v, i) => equalsDeep(v, b[i]));
    } else if (typeof a === 'object' && typeof b === 'object') {
        const keysA = Object.keys(a);
        const keysB = Object.keys(b);
        keysA.sort();
        keysB.sort();

        if (!equalsDeep(keysA, keysB)) {
            return false;
        }

        return keysA.every(k => equalsDeep(a[k as keyof T], b[k as keyof T]));
    }

    return a == b;
}

export const sleep = (ms: number): Promise<void> => {
    return new Promise(resolve => setTimeout(resolve, ms));
}
