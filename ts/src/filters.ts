type Filter = {
    name: string,
    values: string[],
}

export default class Filters {
    readonly filters: Filter[];

    constructor(filterString: string) {
        const splitFilter = (filter: string): Filter => {
            const [name, opts] = filter.trim().split(" ");
            return {
                name,
                values: opts?.split("|") ?? [],
            }
        }

        this.filters = filterString.split(",").map(splitFilter);
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