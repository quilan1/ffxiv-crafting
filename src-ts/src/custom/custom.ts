import CustomInfo, { Id, IdChain, Listing } from './custom_info.js';
import RecStatistics, { RecStatisticsCollection, RecStatisticsSkip } from './rec_statistics.js';

import Elem from '../elem.js';
import Tables, { TableRow } from '../tables.js';
import Filters from '../filters.js';
import Util from '../util.js';
import CheckedTreeControl, { CtcRowData } from '../checked_tree_control.js';

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

let customTreeControl: CheckedTreeControl | undefined;

class Refresh {

    static async refresh() {
        selectors.filters.refresh.disabled = true;

        const filterString = selectors.filters.search.value;

        let info: CustomInfo;
        try {
            info = await CustomInfo.fetch(filterString, true) as CustomInfo;
        } finally {
            selectors.filters.refresh.disabled = false;
        }

        const filteredTopIds = this.getFilteredTopIds(info);

        const parentDiv = document.querySelector('#custom-table-div') as HTMLElement;
        const treeData = this.generateTreeData(info, filteredTopIds);
        const collapsedIds = this.generateCollapsedIds(info, filteredTopIds);

        const headers = ['☑', 'Name', '#/wk', 'Sell', 'Buy', 'Craft', 'Profit'];
        customTreeControl?.destroy();
        customTreeControl = new CheckedTreeControl(parentDiv, headers, treeData, collapsedIds);
        customTreeControl.render();
        customTreeControl.setEventCheck(() => this.displayWorldInfoNew(info, customTreeControl as any));
        this.displayWorldInfoNew(info, customTreeControl);
    }

    private static getFilteredTopIds(info: CustomInfo): Id[] {
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

    private static isItemFiltered(info: CustomInfo, id: Id, filters: Filters): boolean {
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

    private static rowId(id: IdChain) {
        return id.join("-");
    }

    private static generateTreeData(info: CustomInfo, topIds: Id[]): CtcRowData[] {
        const rowData = [];

        const allIds = info.rec_statistics.allChainsOf(topIds);
        const ids = RecStatisticsCollection.filterChains(allIds, id => {
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

            return RecStatisticsSkip.NoSkip;
        });

        const idStats = ids.map(id => ({ id, stats: info.rec_statistics.get(id) })).filter(({ stats }) => stats !== undefined);
        for (const { id, stats } of idStats) {
            const children = stats?.inputs?.childChains(id).map(childId => this.rowId(childId)) ?? [];

            const velocity = stats?.item.statistics.homeworldVelocity?.aq?.toFixed(2) ?? "-";
            const count = (stats?.count ?? 0 > 1) ? `${stats?.count}x ` : '';
            const depth = id.length;
            rowData.push({
                id: this.rowId(id),
                text: `${count}${stats?.item.name}`,
                depth,
                extra: [
                    velocity,
                    stats?.medSellPrice ?? '-',
                    stats?.minBuyPrice ?? "-",
                    stats?.minCraftPrice ?? "-",
                    this.itemProfit(stats),
                ].map(v => v.toString()),
                children,
            });
        }

        return rowData;
    }

    private static generateCollapsedIds(info: CustomInfo, topIds: Id[]): string[] {
        const collapsed = [];
        const allIds = info.rec_statistics.allChainsOf(topIds);
        for (const id of allIds) {
            let stats = info.rec_statistics.get(id);
            if (stats?.isBuyingCheaper === true) {
                collapsed.push(this.rowId(id));
            }
        }
        return collapsed;
    }

    private static displayWorldInfoNew(info: CustomInfo, treeControl: CheckedTreeControl) {
        const worldDiv = Elem.makeDiv({ id: 'custom-world-cur' });
        const ids = treeControl.selectedIds.map(idStr => idStr.split('-').map(id => Number.parseInt(id))) as IdChain[];

        // Enumerate the counts for each item
        const counts: Record<Id, number> = {};
        for (const id of ids) {
            const rowId = this.rowId(id);
            const stats = info.rec_statistics.get(id);
            const item = stats!.item;
            let count = stats!.count;

            const isCollapsed = treeControl.isCollapsed(rowId);

            if (!isCollapsed && item.recipe !== undefined) {
                continue;
            }

            counts[item.item_id] ??= 0;
            counts[item.item_id] += count;
        }

        // Copy the listings
        const listings: Record<Id, Listing[]> = {};
        for (const idStr of Object.keys(counts)) {
            const id = Number.parseInt(idStr);
            listings[id] = Util.cloneDeep(info.item_info[id].listings);
        }

        // build the world info
        type WorldBuyInfo = { item_name: string, name: string, price: number, count: number };
        const worlds: Record<string, Record<string, WorldBuyInfo[]>> = {};
        for (const [idStr, origCount] of Object.entries(counts)) {
            const id = Number.parseInt(idStr);
            let count = origCount;
            for (const listing of listings[id] as Listing[]) {
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
            const dcChildren = [];
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

                dcChildren.push({
                    tag: 'div',
                    children: [
                        { tag: 'div', innerText: `${world}` },
                        { tag: 'div', children: childrenDivs },
                    ]
                });
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
