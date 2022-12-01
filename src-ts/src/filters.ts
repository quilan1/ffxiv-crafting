type Filter = {
    name: string,
    values: string[],
}

export default class Filters {
    readonly filters: Filter[];

    constructor(filterString: string) {
        const splitFilter = (filter: string): Filter => {
            const [name, ...opts] = filter.trim().split(" ");
            return {
                name,
                values: opts?.join(" ")?.split("|") ?? [],
            }
        }

        this.filters = filterString.split(",").map(splitFilter);
    }

    get value(): string {
        const filteredNames = new Set([':count']);
        return this.filters
            .filter(filter => !filteredNames.has(filter.name))
            .map(filter => `${filter.name} ${filter.values.join('|')}`)
            .join(',');
    }

    get(name: string): string[] | undefined {
        return this.filters.find(filter => filter.name === name)?.values;
    }

    getAsInt(name: string): number[] | undefined {
        return this.get(name)?.map(value => Number.parseInt(value));
    }

    getAsFloat(name: string): number[] | undefined {
        return this.get(name)?.map(value => Number.parseFloat(value));
    }

    getOne(name: string): string | undefined {
        return this.get(name)?.[0];
    }

    getOneAsInt(name: string): number | undefined {
        return this.getAsInt(name)?.[0];
    }

    getOneAsFloat(name: string): number | undefined {
        return this.getAsFloat(name)?.[0];
    }
}