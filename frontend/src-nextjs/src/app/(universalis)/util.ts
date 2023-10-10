import { None, OptionType, Some } from "./option";

export type Mutable<T> = {
    -readonly [k in keyof T]: T[k];
};

export type KeysMatching<T extends object, V> = {
    [K in keyof T]-?: T[K] extends V ? K : never
}[keyof T];

export default class Util {
    static keysOf<T extends Record<K, unknown>, K extends string | number | symbol>(o: T): (keyof T)[] {
        return Object.keys(o) as (keyof T)[];
    };

    static entriesOf<T extends Record<K, unknown>, K extends string | number | symbol>(o: T): [keyof T, T[keyof T]][] {
        return Object.entries(o) as [keyof T, T[keyof T]][];
    };

    static sorted<T>(_arr: Iterable<T>): T[] {
        const arr = [..._arr];
        arr.sort();
        return arr;
    }

    static tryParse(s: string): OptionType<number> {
        const v = parseInt(s);
        return Number.isNaN(v) ? None() : Some(v);
    }

    static equals<T>(a: T[], b: T[]) {
        return a.length === b.length && a.every((v, i) => v === b[i]);
    }

    static cloneDeep<T>(obj: T): T {
        if (typeof obj === 'string' || typeof obj === 'number' || obj === null || obj === undefined) {
            return obj;
        } else if (Array.isArray(obj)) {
            return obj.map(value => this.cloneDeep(value as unknown)) as T;
        } else if (typeof obj === 'object') {
            const ret = {} as T;
            for (const [key, value] of Object.entries(obj as object)) {
                // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
                ret[key as keyof T] = this.cloneDeep(value);
            }
            return ret;
        } else {
            return obj;
        }
    }

    static equalsDeep<T>(a: T, b: T): boolean {
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
            return a.length === b.length && a.every((v, i) => this.equalsDeep(v, b[i]));
        } else if (typeof a === 'object' && typeof b === 'object') {
            const keysA = Object.keys(a);
            const keysB = Object.keys(b);
            keysA.sort();
            keysB.sort();

            if (!Util.equalsDeep(keysA, keysB)) {
                return false;
            }

            return keysA.every(k => this.equalsDeep(a[k as keyof T], b[k as keyof T]));
        }

        return a == b;
    }

    static dataCenter(world: string): string {
        return {
            'Halicarnassus': 'Dynamis',
            'Maduin': 'Dynamis',
            'Marilith': 'Dynamis',
            'Seraph': 'Dynamis',

            'Adamantoise': 'Aether',
            'Cactuar': 'Aether',
            'Faerie': 'Aether',
            'Gilgamesh': 'Aether',
            'Jenova': 'Aether',
            'Midgardsormr': 'Aether',
            'Sargatanas': 'Aether',
            'Siren': 'Aether',

            'Balmung': 'Crystal',
            'Brynhildr': 'Crystal',
            'Coeurl': 'Crystal',
            'Diabolos': 'Crystal',
            'Goblin': 'Crystal',
            'Malboro': 'Crystal',
            'Mateus': 'Crystal',
            'Zalera': 'Crystal',

            'Behemoth': 'Primal',
            'Excalibur': 'Primal',
            'Exodus': 'Primal',
            'Famfrit': 'Primal',
            'Hyperion': 'Primal',
            'Lamia': 'Primal',
            'Leviathan': 'Primal',
            'Ultros': 'Primal',
        }[world] ?? "<UNKNOWN>" as string;
    }

    static sleep(ms: number) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}
