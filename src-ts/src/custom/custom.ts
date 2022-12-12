import CustomInfo, { Id, IdChain, Listing } from './custom_info.js';
import RecStatistics, { RecStatisticsCollection, RecStatisticsSkip } from './rec_statistics.js';

import Elem from '../elem.js';
import Filters from '../filters.js';
import Util from '../util.js';
import CheckedTreeControl, { CtcRowData } from '../checked_tree_control.js';

const savedFilters: any[] = [];

const _regexNameSkip = /(Fire|Ice|Wind|Earth|Lightning|Water) (Shard|Crystal|Cluster)/;

class CustomDlg {
    customTreeControl: CheckedTreeControl | null;
    info: CustomInfo | null;
    filteredTopIds: Id[] | null;

    constructor() {
        this.customTreeControl = null;
        this.info = null;
        this.filteredTopIds = null;
    }

    setupEvents() {
        FiltersDlg.setupEvents(this);
    }

    async onRefreshClick(debug?: boolean) {
        await FiltersDlg.withDisabledRefresh(async () => {
            const fn = (status: string) => FiltersDlg.setStatus(status);
            if (debug === true) {
                this.info = await CustomInfo.fetch(FiltersDlg.searchValue, FiltersDlg.countValue, FiltersDlg.dataCenter, debug === true) as CustomInfo;
            } else {
                this.info = await CustomInfo.fetchLazy(FiltersDlg.searchValue, FiltersDlg.countValue, FiltersDlg.dataCenter, fn) as CustomInfo;
            }
        });

        this.filteredTopIds = this.getFilteredTopIds();

        const parentDiv = document.querySelector('#custom-table-div') as HTMLElement;
        const treeData = this.generateTreeData(this.filteredTopIds);
        const collapsedIds = this.generateCollapsedIds(this.filteredTopIds);

        const headers = ['☑', 'Name', '#/day', '#/wk', '#/2wk', 'Sell', 'Buy', 'Craft', 'Profit'];
        this.customTreeControl?.destroy();
        this.customTreeControl = new CheckedTreeControl(parentDiv, headers, treeData, collapsedIds);
        this.customTreeControl.render();
        this.customTreeControl.setEventCheck(() => this.displayWorldInfoNew());
        this.displayWorldInfoNew();
    }

    /////////////////////////////////////////////////

    private static rowId(id: IdChain) {
        return id.join("-");
    }

    private getFilteredTopIds(): Id[] {
        if (!this.info) return [];

        const filters = new Filters(FiltersDlg.searchValue);

        let filteredIds = [];
        for (const id of this.info.top_ids) {
            const item = this.info.item_info[id];
            if (item === undefined) {
                continue;
            }

            if (!this.isItemFiltered(id, filters)) {
                continue;
            }

            filteredIds.push(id);
        }

        filteredIds.sort((a, b) => this.itemProfit(this.info?.rec_statistics.get(b)) - this.itemProfit(this.info?.rec_statistics.get(a)));

        const limitFilter = filters.getOneAsInt(":limit");
        if (limitFilter === undefined) {
            return filteredIds;
        }

        return filteredIds.slice(0, limitFilter);
    }

    private isItemFiltered(id: Id, filters: Filters): boolean {
        if (!this.info) {
            return true;
        }

        const minVelocity = filters.getOneAsFloat(":min_velocity");
        if (minVelocity === undefined) {
            return true;
        }

        const velocity = this.info.item_info[id].statistics?.homeworldVelocityWeek?.aq;
        if (velocity === undefined) {
            return false;
        }

        return velocity >= minVelocity;
    }

    private itemProfit(stats?: RecStatistics): number {
        if (stats?.medSellPrice === undefined) {
            const value = stats?.buyCraftPrice;
            return (value !== undefined) ? -value : Number.MIN_SAFE_INTEGER;
        }
        return stats?.profit ?? Number.MIN_SAFE_INTEGER;
    }

    private generateTreeData(topIds: Id[]): CtcRowData[] {
        if (!this.info) return [];

        const rowData = [];

        const allIds = this.info.rec_statistics.allChainsOf(topIds);
        const ids = RecStatisticsCollection.filterChains(allIds, id => {
            const [item_id] = id.slice(-1);
            const item = this.info?.item_info[item_id];
            if (item === undefined) {
                return RecStatisticsSkip.SkipEverything;
            }
    
            let stats = this.info?.rec_statistics.get(id);
            if (stats === undefined) {
                return RecStatisticsSkip.SkipEverything;
            }
    
            if (_regexNameSkip.test(item.name)) {
                return RecStatisticsSkip.SkipEverything;
            }

            return RecStatisticsSkip.NoSkip;
        });

        const idStats = ids.map(id => ({ id, stats: this.info?.rec_statistics.get(id) })).filter(({ stats }) => stats !== undefined);
        for (const { id, stats } of idStats) {
            const children = stats?.inputs?.childChains(id).map(childId => CustomDlg.rowId(childId)) ?? [];

            const statistics = stats?.item.statistics;
            const velocity1 = statistics?.homeworldVelocityDay?.aq?.toFixed(2) ?? '-';
            const velocity7 = statistics?.homeworldVelocityWeek?.aq?.toFixed(2) ?? '-';
            const velocity14 = statistics?.homeworldVelocityWeeks?.aq?.toFixed(2) ?? '-';
            const count = (stats?.count ?? 0 > 1) ? `${stats?.count}x ` : '';
            const depth = id.length;
            rowData.push({
                id: CustomDlg.rowId(id),
                depth,
                children,
                text: [
                    `${count}${stats?.item.name}`,
                    velocity1,
                    velocity7,
                    velocity14,
                    stats?.medSellPrice ?? '-',
                    stats?.minBuyPrice ?? "-",
                    stats?.minCraftPrice ?? "-",
                    this.itemProfit(stats),
                ].map(v => v.toString()),
            });
        }

        return rowData;
    }

    private generateCollapsedIds(topIds: Id[]): string[] {
        if (!this.info) return [];

        const collapsed = [];
        const allIds = this.info.rec_statistics.allChainsOf(topIds);
        for (const id of allIds) {
            let stats = this.info.rec_statistics.get(id);
            if (stats?.isBuyingCheaper === true) {
                collapsed.push(CustomDlg.rowId(id));
            }
        }
        return collapsed;
    }

    /////////////////////////////////////////////////

    private displayWorldInfoNew() {
        if (!this.info || !this.customTreeControl) return;

        const worldDiv = Elem.makeDiv({ id: 'custom-world-cur' });
        const ids = this.customTreeControl.selectedIds.map(idStr => idStr.split('-').map(id => Number.parseInt(id))) as IdChain[];

        // Enumerate the counts for each item
        const counts: Record<Id, number> = {};
        for (const id of ids) {
            const rowId = CustomDlg.rowId(id);
            const stats = this.info.rec_statistics.get(id);
            const item = stats!.item;
            let count = stats!.count;

            const isCollapsed = this.customTreeControl.isCollapsed(rowId);

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
            listings[id] = Util.cloneDeep(this.info.item_info[id].listings);
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
                    item_name: this.info.item_info[id].name,
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

        this.selectors.replaceWorld(worldDiv);
    }

    private get selectors() {
        return {
            get world() {
                return document.querySelector('#custom-world-cur');
            },
            get table() {
                return document.querySelector('#custom-table-cur');
            },
            replaceWorld(world: HTMLElement) {
                this.world?.parentElement?.replaceChild(world, this.world);
            }
        };
    }

    onCountChange() {
        if (!this.info || !this.customTreeControl || !this.filteredTopIds) return;

        try {
            const count = FiltersDlg.countValue;
            if (count <= 0) return;
            this.info.calcRecStatistics(count);
            this.customTreeControl.updateRows(this.generateTreeData(this.filteredTopIds));
        } catch(_) {}
    }
};

class FiltersDlg {
    static setupEvents(customDlg: CustomDlg) {
        this.selectors.refresh.onclick = (e) => customDlg.onRefreshClick(e.shiftKey);
        this.selectors.load.onclick = () => this.load();
        this.selectors.save.onclick = () => this.save();
        this.selectors.saveAs.onclick = () => this.saveAs();
        this.selectors.delete.onclick = () => this.delete();
        this.selectors.list.onchange = () => this.changeSelection();
        this.selectors.count.onchange = () => customDlg.onCountChange();
        this.loadFromStorage();
        this.changeSelection();
    }

    private static save() {
        const list = this.selectors.list;
        if (list.selectedIndex != 0) {
            const value = this.selectors.search.value;
            savedFilters[list.selectedIndex - 1].value = value;
            list.options[list.selectedIndex].value = value;
        }
        this.changeSelection();
    }

    private static saveAs() {
        const value = this.selectors.search.value;
        const saveName = prompt('With what name would you like to save this filter as?', value);
        if (saveName === null) {
            return;
        }

        savedFilters.push({ name: saveName, filter: value });
        this.addOption(saveName, value);

        const list = this.selectors.list;
        list.selectedIndex = list.options.length - 1;
        this.changeSelection();
    }

    private static addOption(innerText: string, value: string): HTMLOptionElement {
        const option = Elem.makeOption({ innerText: innerText, value: value });
        const list = this.selectors.list;
        list.add(option);
        return option;
    }

    private static load() {
        const curSelected = this.selectors.list.value;
        if (curSelected) {
            const filters = new Filters(curSelected);
            this.selectors.count.value = filters.getOne(':count') ?? "1";
            this.selectors.search.value = filters.value;
        }

        this.changeSelection();
    }

    private static delete() {
        const list = this.selectors.list;
        if (list.selectedIndex != 0) {
            savedFilters.splice(list.selectedIndex - 1, 1);
            list.options.remove(list.selectedIndex);
        }

        this.changeSelection();
    }

    private static loadFromStorage() {
        const storageFilters = localStorage.getItem('custom-filters');
        if (storageFilters) {
            savedFilters.splice(0, savedFilters.length);
            savedFilters.push(...JSON.parse(storageFilters));
            for (const { name, filter } of savedFilters) {
                this.addOption(name, filter);
            }
        }
    }

    private static changeSelection() {
        if (this.selectors.list.selectedIndex == 0) {
            this.selectors.save.disabled = true;
            this.selectors.delete.disabled = true;
        } else {
            this.selectors.save.disabled = false;
            this.selectors.delete.disabled = false;
        }

        localStorage.setItem('custom-filters', JSON.stringify(savedFilters));
    }

    private static get selectors() {
        return {
            get search() { return document.querySelector('#custom-filters') as HTMLInputElement; },
            get list() { return document.querySelector('#custom-filters-list') as HTMLSelectElement; },
            get load() { return document.querySelector('#custom-filters-load') as HTMLButtonElement; },
            get save() { return document.querySelector('#custom-filters-save') as HTMLButtonElement; },
            get saveAs() { return document.querySelector('#custom-filters-save-as') as HTMLButtonElement; },
            get delete() { return document.querySelector('#custom-filters-delete') as HTMLButtonElement; },
            get refresh() { return document.querySelector('#custom-filters-refresh') as HTMLButtonElement; },
            get count() { return document.querySelector('#custom-count') as HTMLInputElement; },
            get dataCenter() { return document.querySelector('#custom-data-center') as HTMLSelectElement; },
            get status() { return document.querySelector('#custom-filters-status') as HTMLElement; },
        }
    }

    static get searchValue(): string {
        return this.selectors.search.value;
    }

    static get countValue(): number {
        return Number.parseInt(this.selectors.count.value);
    }

    static get dataCenter(): string {
        return this.selectors.dataCenter.value;
    }

    static async withDisabledRefresh(func: () => Promise<void>): Promise<void> {
        this.selectors.refresh.disabled = true;

        try {
            await func();
        } finally {
            this.selectors.refresh.disabled = false;
        }
    }

    static setStatus(status: string) {
        this.selectors.status.innerText = status;
    }
};

export default CustomDlg;
