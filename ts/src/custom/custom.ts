import CustomInfo, { Listing } from './custom_info.js';
import RecStatistics, { RecStatisticsCollection, RecStatisticsSkip } from './rec_statistics.js';

import Elem from '../elem.js';
import Tables, { TableRow } from '../tables.js';
import Filters from '../filters.js';
import Util from '../util.js';

interface ICustomDlgInfo extends CustomInfo {
    collapsed: Record<string, boolean>,
    isCollapsed(id: number[]): boolean | undefined,
    rowId(id: number[]): string,
}

let selectors: Selectors;
const savedFilters: any[] = [];

type SelectorFilters = {
    search: HTMLInputElement,
    list: HTMLSelectElement,
    load: HTMLButtonElement,
    save: HTMLButtonElement,
    saveAs: HTMLButtonElement,
    delete: HTMLButtonElement,
    refresh: HTMLButtonElement,
}

type SelectorData = {
    cur: () => HTMLElement,
    tbody: () => HTMLElement,
    set: (_new: HTMLElement) => void,
    setBody: (_new: HTMLElement) => void,
}

type Selectors = {
    filters: SelectorFilters,
    data: {
        table: SelectorData,
        world: SelectorData,
    },
    row: (id: string) => HTMLTableRowElement | null,
    rowCheck: (id: string) => HTMLInputElement | null,
    rowPlus: (id: string) => HTMLButtonElement | null,
};

const _getSelectors = () => {
    const makeData = (_: string, child: string) => {
        return {
            cur: () => document.querySelector(child) as HTMLElement,
            tbody: () => document.querySelector(`${child} > tbody`) as HTMLElement,
            set(_new: HTMLElement) {
                const me = this.cur();
                me?.parentNode?.replaceChild(_new, me);
            },
            setBody(_new: HTMLElement) {
                const me = this.tbody();
                me?.parentNode?.replaceChild(_new, me);
            },
        }
    };

    return {
        filters: {
            search: document.querySelector('#custom-filters') as HTMLInputElement,
            list: document.querySelector('#custom-filters-list') as HTMLSelectElement,
            load: document.querySelector('#custom-filters-load') as HTMLButtonElement,
            save: document.querySelector('#custom-filters-save') as HTMLButtonElement,
            saveAs: document.querySelector('#custom-filters-save-as') as HTMLButtonElement,
            delete: document.querySelector('#custom-filters-delete') as HTMLButtonElement,
            refresh: document.querySelector('#custom-filters-refresh') as HTMLButtonElement,
        },
        data: {
            table: makeData('#custom-table-div', '#custom-table-cur'),
            world: makeData('#custom-world-div', '#custom-world-cur'),
        },
        row: (id: string) => document.querySelector(`table#custom-table-cur > tbody > #${id}`) as HTMLTableRowElement | null,
        rowCheck: (id: string) => selectors!.row(id)?.querySelector(`:scope > td:nth-child(1) > input[type='checkbox']`) as HTMLInputElement | null,
        rowPlus: (id: string) => selectors!.row(id)?.querySelector(`:scope > td:nth-child(2) > button`) as HTMLButtonElement | null,
    };
};

const _regexNameSkip = /(Fire|Ice|Wind|Earth|Lightning|Water) (Shard|Crystal|Cluster)/;

class Custom {
    static setupEvents() {
        selectors = _getSelectors();
        selectors.filters.refresh.onclick = (_: any) => Refresh.refresh();
        selectors.filters.load.onclick = (_: any) => FiltersDlg.load();
        selectors.filters.save.onclick = (_: any) => FiltersDlg.save();
        selectors.filters.saveAs.onclick = (_: any) => FiltersDlg.saveAs();
        selectors.filters.delete.onclick = (_: any) => FiltersDlg.delete();
        selectors.filters.list.onchange = (_: any) => FiltersDlg.changeSelection();
        FiltersDlg.loadFromStorage();
        FiltersDlg.changeSelection();
    }
};

class Refresh {
    static async refresh() {
        selectors.filters.refresh.disabled = true;

        const filterString = selectors.filters.search.value;

        let info: ICustomDlgInfo;
        try {
            info = await CustomInfo.fetch(filterString) as ICustomDlgInfo;
        } finally {
            selectors.filters.refresh.disabled = false;
        }

        this.initInfo(info);
        const filteredTopIds = this.getFilteredTopIds(info);
        const filteredChainIds = this.getFilteredChainIds(info, filteredTopIds);

        const rows = filteredChainIds.map(chainId => this.makeRow(info, chainId)).filter(row => row !== undefined) as any as TableRow[];
        selectors.data.table.setBody(Tables.createRows(rows));

        this.setupTableTriggers(info, info.rec_statistics.allChains());
    }

    private static initInfo(info: ICustomDlgInfo) {
        info.collapsed = {};
        info.isCollapsed = (id: number[]): boolean | undefined => info.collapsed[info.rowId(id)];
        info.rowId = (id: number[]): string => `row-${id.join("-")}`;
    }

    private static getFilteredTopIds(info: ICustomDlgInfo) {
        const filters = new Filters(selectors.filters.search.value);

        let filteredIds = [];
        for (const id of info.top_ids) {
            const item = info.item_info[id];
            if (item === undefined) {
                continue;
            }

            if (!this.isItemFiltered(info, id, filters)) {
                continue;
            }

            filteredIds.push(id);
        }

        filteredIds.sort((a, b) => this.itemProfit(info.rec_statistics.get(b)) - this.itemProfit(info.rec_statistics.get(a)));

        const limitFilter = filters.getOneAsInt(":limit");
        if (limitFilter === undefined) {
            return filteredIds;
        }

        return filteredIds.slice(0, limitFilter);
    }

    private static getFilteredChainIds(info: ICustomDlgInfo, filteredTopIds: number[]) {
        const ids = info.rec_statistics.allChainsOf(filteredTopIds);
        return RecStatisticsCollection.filterChains(ids, id => this.basicChainFilter(info, id));
    }

    private static basicChainFilter(info: ICustomDlgInfo, id: number[]): RecStatisticsSkip {
        const [item_id] = id.slice(-1);
        const item = info.item_info[item_id];
        if (item === undefined) {
            return RecStatisticsSkip.SkipEverything;
        }

        let stats = info.rec_statistics.get(id);
        if (stats === undefined) {
            return RecStatisticsSkip.SkipEverything;
        }

        if (_regexNameSkip.test(item.name)) {
            return RecStatisticsSkip.SkipEverything;
        }

        return stats.isBuyingCheaper ? RecStatisticsSkip.SkipChildren : RecStatisticsSkip.NoSkip;
    }

    private static isItemFiltered(info: ICustomDlgInfo, id: number, filters: Filters): boolean {
        const minVelocity = filters.getOneAsFloat(":min_velocity");
        if (minVelocity === undefined) {
            return true;
        }

        const velocity = info.item_info[id].statistics?.homeworldVelocity?.aq;
        if (velocity === undefined) {
            return false;
        }

        return velocity >= minVelocity;
    }

    private static itemProfit(stats?: RecStatistics): number {
        if (stats?.medSellPrice === undefined) {
            const value = stats?.buyCraftPrice;
            return (value !== undefined) ? -value : Number.MIN_SAFE_INTEGER;
        }
        return stats?.profit ?? Number.MIN_SAFE_INTEGER;
    }

    //////////////////////////////

    private static makeRow(info: ICustomDlgInfo, id: number[]): TableRow | undefined {
        const stats = info.rec_statistics.get(id);
        if (stats === undefined) {
            return undefined;
        }

        const velocity = stats.item.statistics.homeworldVelocity?.aq?.toFixed(2) ?? "--";

        const _name = (v: string) => this.nameFieldCallback(info, id, v);
        const _check = (td: HTMLTableCellElement) => td.appendChild(Elem.makeCheckbox());

        const _depth = (stats.count > 1) ? `${stats.count}x ` : '';
        return [
            _check,
            _name(`${_depth}${stats.item.name} [${velocity}]`),
            stats?.medSellPrice ?? "-",
            stats?.minBuyPrice ?? "-",
            stats?.minCraftPrice ?? "-",
            this.itemProfit(stats),
        ];
    }

    private static nameFieldCallback(info: ICustomDlgInfo, id: number[], textValue: string) {
        const stats = info.rec_statistics.get(id);
        const item = stats?.item;
        const hasRecipe = item?.recipe !== undefined;
        const depth = id.length;

        return (td: HTMLTableCellElement, tr: HTMLTableRowElement) => {
            tr.id = info.rowId(id);

            td.className = "item-name";
            td.style.paddingLeft = `${20 * (depth - 1) + (hasRecipe ? 0 : 24)}px`;
            td.innerText = textValue;

            if (!hasRecipe) {
                return;
            }

            const isCollapsed = info.isCollapsed(id) ?? stats?.isBuyingCheaper ?? true;
            const plus = Elem.makeButton({ innerText: ['-', '+'][Number(isCollapsed)], className: 'collapsable' });
            td.prepend(plus);
        }
    }

    private static toggleExpandChildren(info: ICustomDlgInfo, id: number[], collapse: boolean) {
        const stats = info.rec_statistics.get(id);
        if (stats === undefined) {
            return;
        }

        const rowId = info.rowId(id);
        const children = stats.inputs?.allChains(id) ?? [];
        info.collapsed[rowId] = collapse;

        if (collapse) {
            for (const child of children) {
                const childNode = selectors.row(info.rowId(child));
                childNode?.parentNode?.removeChild(childNode);
            }
            return;
        }

        const filteredChildren = RecStatisticsCollection.filterChains(children, id => {
            const isCollapsed = info.isCollapsed(id);
            if (isCollapsed !== undefined) {
                return isCollapsed ? RecStatisticsSkip.SkipChildren : RecStatisticsSkip.NoSkip;
            }

            return this.basicChainFilter(info, id);
        });

        // Create the new rows
        const thisRow = selectors.row(rowId);
        const rows = filteredChildren.map(id => this.makeRow(info, id))
            .filter(row => row !== undefined)
            .map(row => Tables.createRow(row as TableRow));

        // Stick them onto the page
        thisRow?.after(...rows);

        // Propogate the checkmarks down to the newly created children
        const checkbox = selectors.rowCheck(rowId);
        if (checkbox) {
            for (const child of filteredChildren) {
                const childCheckbox = selectors.rowCheck(info.rowId(child));
                if (!childCheckbox) {
                    continue;
                }

                childCheckbox.checked = checkbox.checked;
            }
        }
    }

    private static toggleCheckChildren(info: ICustomDlgInfo, id: number[], value: boolean) {
        const stats = info.rec_statistics.get(id);
        if (stats === undefined) {
            return;
        }

        const children = stats.inputs?.allChains(id) ?? [];

        for (const child of children) {
            const check = selectors.rowCheck(info.rowId(child)) as HTMLInputElement | null;
            if (check) {
                check.checked = value;
            }
        }
    }

    private static setupTableTriggers(info: ICustomDlgInfo, ids: number[][]) {
        for (const id of ids) {
            const rowId = info.rowId(id);
            if (!selectors.row(rowId)) {
                continue;
            }

            const checkbox = selectors.rowCheck(rowId);
            const plus = selectors.rowPlus(rowId);

            if (plus) {
                plus.onclick = _ => {
                    const toCollapse = plus.innerText === '-';
                    plus.innerText = ['-', '+'][Number(toCollapse)];
                    this.toggleExpandChildren(info, id, toCollapse);
                    this.setupTableTriggers(info, ids);
                    this.displayWorldInfo(info, ids);
                }
            }

            if (checkbox) {
                checkbox.onclick = _ => {
                    this.toggleCheckChildren(info, id, checkbox.checked);
                    this.displayWorldInfo(info, ids);
                }
            }
        }
    }

    private static displayWorldInfo(info: ICustomDlgInfo, ids: number[][]) {
        const worldDiv = Elem.makeDiv({ id: 'custom-world-cur' });

        // Enumerate the counts for each item
        const counts = new Map<number, number>();
        for (const id of ids) {
            const check = selectors.rowCheck(info.rowId(id));
            if (!check || check.checked === false) {
                continue;
            }

            const stats = info.rec_statistics.get(id);
            const item = stats!.item;
            let count = stats!.count;

            const isCollapsed = info.isCollapsed(id) ?? stats?.isBuyingCheaper ?? true;

            if (!isCollapsed && item.recipe !== undefined) {
                continue;
            }

            if (!counts.has(item.item_id)) {
                counts.set(item.item_id, 0);
            }
            counts.set(item.item_id, counts.get(item.item_id) as number + count);
        }

        // Copy the listings
        const listings = new Map<number, Listing[]>();
        for (const id of counts.keys()) {
            listings.set(id, Util.cloneDeep(info.item_info[id].listings));
        }

        // build the world info
        type WorldBuyInfo = { item_name: string, name: string, price: number, count: number };
        const worlds: Record<string, Record<string, WorldBuyInfo[]>> = {};
        for (const [id, origCount] of counts.entries()) {
            let count = origCount;
            for (const listing of listings.get(id) as Listing[]) {
                if (count <= 0) {
                    break;
                }

                if (listing.count === 0) {
                    continue;
                }

                const usedCount = listing.count;
                listing.count -= usedCount;
                count -= usedCount;

                const dataCenter = Util.dataCenter(listing.world);

                worlds[dataCenter] ??= {};
                worlds[dataCenter][listing.world] ??= [];
                worlds[dataCenter][listing.world].push({
                    item_name: info.item_info[id].name,
                    name: listing.name,
                    price: Math.floor(listing.price / 1.05),
                    count: usedCount,
                });
            }
        }

        for (const [dataCenter, worldsInfo] of Object.entries(worlds)) {
            const dcTitleDiv = Elem.makeDiv({ innerText: dataCenter });
            const dcChildren = [];// as (HTMLElement)[];
            for (const [world, worldInfo] of Object.entries(worldsInfo)) {
                const childrenDivs = worldInfo.map(({ item_name, name, price, count }) => {
                    return {
                        tag: 'div',
                        children: [
                            { tag: 'div', innerText: `${count}x` },
                            { tag: 'div', innerText: `[${price} gil]` },
                            { tag: 'div', innerText: `${item_name} [${name}]` },
                        ]
                    };
                });

                // const titleDiv = Elem.makeDiv({ innerText: `${world} [${dataCenter}]` });
                // const childrenDiv = Elem.makeDiv({ children: childrenDivs });
                // const curWorldDiv = Elem.makeDiv({ children: [titleDiv, childrenDiv] });
                // worldDiv.appendChild(curWorldDiv);

                const curWorldDiv = {
                    tag: 'div',
                    children: [
                        { tag: 'div', innerText: `${world}` },
                        { tag: 'div', children: childrenDivs },
                    ]
                };
                // const titleDiv = Elem.makeDiv({ innerText: `${world} [${dataCenter}]` });
                // const childrenDiv = Elem.makeDiv({ children: childrenDivs });
                // const curWorldDiv = Elem.makeDiv({ children: [titleDiv, childrenDiv] });
                dcChildren.push(curWorldDiv);
            }
            const dcDiv = Elem.makeDiv({ children: [dcTitleDiv, { tag: 'div', children: dcChildren }] });
            worldDiv.appendChild(dcDiv);
        }

        selectors.data.world.set(worldDiv);
    }
}

class FiltersDlg {
    static save() {
        const list = selectors.filters.list;
        if (list.selectedIndex != 0) {
            const value = selectors.filters.search.value;
            savedFilters[list.selectedIndex - 1].value = value;
            list.options[list.selectedIndex].value = value;
        }
        this.changeSelection();
    }

    static saveAs() {
        const value = selectors.filters.search.value;
        const saveName = prompt('With what name would you like to save this filter as?', value);
        if (saveName === null) {
            return;
        }

        savedFilters.push({ name: saveName, filter: value });
        this.addOption(saveName, value);

        const list = selectors.filters.list;
        list.selectedIndex = list.options.length - 1;
        this.changeSelection();
    }

    private static addOption(innerText: string, value: string): HTMLOptionElement {
        const option = Elem.makeOption({ innerText: innerText, value: value });
        const list = selectors.filters.list;
        list.add(option);
        return option;
    }

    static load() {
        const curSelected = selectors.filters.list.value;
        if (curSelected) {
            selectors.filters.search.value = curSelected;
        }

        this.changeSelection();
    }

    static delete() {
        const list = selectors.filters.list;
        if (list.selectedIndex != 0) {
            savedFilters.splice(list.selectedIndex - 1, 1);
            list.options.remove(list.selectedIndex);
        }

        this.changeSelection();
    }

    static loadFromStorage() {
        const storageFilters = localStorage.getItem('custom-filters');
        if (storageFilters) {
            savedFilters.splice(0, savedFilters.length);
            savedFilters.push(...JSON.parse(storageFilters));
            for (const { name, filter } of savedFilters) {
                this.addOption(name, filter);
            }
        }
    }

    static changeSelection() {
        if (selectors.filters.list.selectedIndex == 0) {
            selectors.filters.save.disabled = true;
            selectors.filters.delete.disabled = true;
        } else {
            selectors.filters.save.disabled = false;
            selectors.filters.delete.disabled = false;
        }

        localStorage.setItem('custom-filters', JSON.stringify(savedFilters));
    }
};

export default Custom;
