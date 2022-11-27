export type Mutable<T> = {
    -readonly [k in keyof T]: T[k];
};

export default class Util {
    private static readonly WEBPAGE: string = "http://127.0.0.1:3001";

    static fetchPage(fileName: string): Promise<Response> {
        return this.fetch(`web/${fileName}`);
    }

    static fetch(path: string, args?: object): Promise<Response> {
        return window.fetch(`${this.WEBPAGE}/${path}`, args);
    }

    static sorted<T>(_arr: Iterable<T>): T[] {
        const arr = [..._arr];
        arr.sort();
        return arr;
    }

    static fixFlexOverflow() {
        const overflowElems = document.querySelectorAll('.table-scroll');
        for (const elem of overflowElems) {
            let node = elem.parentNode as HTMLElement;
            while (node) {
                if (node.tagName === 'BODY') break;
                node.style.minHeight = '0px';
                node = node.parentNode as HTMLElement;
            }
        }
    }

    static equals<T>(a: T[], b: T[]) {
        return a.length === b.length && a.every((v, i) => v === b[i]);
    }

    static cloneDeep<T>(obj: T): T {
        if (typeof obj === 'string' || typeof obj === 'number' || obj === null || obj === undefined) {
            return obj;
        } else if (Array.isArray(obj)) {
            return obj.map(value => Util.cloneDeep(value)) as T;
        } else if (typeof obj === 'object') {
            const ret = {} as any;
            for (const [key, value] of Object.entries(obj as object)) {
                ret[key] = this.cloneDeep(value);
            }
            return ret;
        } else {
            return obj;
        }
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
        }[world] as string;
    }
}
