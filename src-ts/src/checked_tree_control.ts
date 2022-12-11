import Elem from "./elem.js";
import Tables, { TableRow } from "./tables.js";
import Util from "./util.js";

export type CtcRowData = {
    id: string,
    children: string[],
    depth: number,
    text: string[],
};

export default class CheckedTreeControl {
    private parent: HTMLElement;
    private headers: string[];
    private data: CtcRowData[];
    private collapsedIds: Set<string>;
    private table: HTMLElement | null;
    private eventCheck: (() => void) | null;

    constructor(tableBody: HTMLElement, headers: string[], data: CtcRowData[], collapsed?: string[]) {
        this.parent = tableBody;
        this.headers = headers;
        this.data = data;
        this.collapsedIds = new Set(collapsed);
        this.table = null;
        this.eventCheck = null;
    }

    setEventCheck(func: () => void) {
        this.eventCheck = func;
    }

    render() {
        const rows = this.renderedRows(this.data);
        const table = Tables.createTable(this.headers, rows);

        if (this.table) {
            this.parent.replaceChild(table, this.table);
        } else {
            this.parent.appendChild(table);
        }
        this.table = table;
        this.setupTableTriggers();
    }

    destroy() {
        if (this.table) {
            this.parent.removeChild(this.table);
        }
    }

    updateRows(data: CtcRowData[]) {
        this.data = data;
        for (const rowData of this.data) {
            for (let index=0; index < rowData.text.length; ++index) {
                const cell = this.selectors.cell(rowData.id, index);
                if (cell !== undefined) {
                    cell.innerText = rowData.text[index];
                }
            }
        }
    }

    get selectors() {
        const table = this.table;
        return {
            row(id: string): HTMLTableRowElement | undefined {
                return table?.querySelector(`:scope > tbody > tr[ctcrowid="${id}"]`) as HTMLTableRowElement | undefined;
            },
            check(id: string): HTMLInputElement | undefined {
                return this.row(id)?.querySelector(':scope > td:nth-child(1) > input[type="checkbox"]') as HTMLInputElement | undefined;
            },
            plus(id: string): HTMLButtonElement | undefined {
                return this.row(id)?.querySelector(':scope > td:nth-child(2) > button') as HTMLButtonElement | undefined;
            },
            cell(id: string, index: number): HTMLElement | undefined {
                if (index === 0) {
                    return this.row(id)?.querySelector(':scope > td:nth-child(2) > span') as HTMLElement | undefined;
                } else {
                    return this.row(id)?.querySelector(`:scope > td:nth-child(${index+2})`) as HTMLElement | undefined;
                }
            }
        }
    }

    get selectedIds(): string[] {
        const ids = [];
        for (const rowData of this.data) {
            const check = this.selectors.check(rowData.id);
            if (check?.checked === true) {
                ids.push(rowData.id);
            }
        }
        return ids;
    }

    isCollapsed(id: string): boolean {
        return this.collapsedIds.has(id);
    }

    ////////////////////////////////////////////////////////////

    private makeRow(data: CtcRowData): TableRow {
        const _check = (td: HTMLTableCellElement, tr: HTMLTableRowElement) => {
            tr.setAttribute('ctcRowId', data.id);
            td.appendChild(Elem.makeCheckbox());
        }

        const _text = (td: HTMLTableCellElement) => {
            td.className = "item-name";
            td.style.paddingLeft = `${20 * (data.depth - 1) + (data.children.length > 0 ? 0 : 24)}px`;

            if (data.children.length > 0) {
                const isCollapsed = this.collapsedIds.has(data.id);
                td.append(Elem.makeButton({ innerText: ['-', '+'][Number(isCollapsed)], className: 'collapsable' }));
            }
            td.append(Elem.makeSpan({ innerText: data.text[0] }));
        }

        return [
            _check,
            _text,
            ...data.text.slice(1)
        ];
    }

    private renderedRows(dataRows: CtcRowData[]): TableRow[] {
        const rows = [];
        let skipParent = undefined;
        for (const row of dataRows) {
            if (skipParent !== undefined && row.id.startsWith(skipParent)) {
                continue;
            } else {
                skipParent = undefined;
            }

            if (this.collapsedIds.has(row.id)) {
                skipParent = `${row.id}-`;
            }

            rows.push(this.makeRow(row));
        }

        return rows;
    }

    ////////////////////////////////////////////////////////////

    private setupTableTriggers() {
        const triggerEventCheck = () => {
            if (this.eventCheck) {
                this.eventCheck();
            }
        }

        for (const row of this.data) {
            const plus = this.selectors.plus(row.id);
            if (plus) {
                plus.onclick = _ => {
                    const toCollapse = plus.innerText === '-';
                    plus.innerText = ['-', '+'][Number(toCollapse)];
                    this.toggleChildrenPlus(row.id, toCollapse);
                    this.setupTableTriggers();
                    triggerEventCheck();
                }
            }

            const checkbox = this.selectors.check(row.id);
            if (checkbox) {
                checkbox.onclick = _ => {
                    this.toggleChildrenCheck(row.id, checkbox.checked);
                    triggerEventCheck();
                }
            }
        }
    }

    private toggleChildrenPlus(id: string, collapse: boolean) {
        const rowData = this.data.find(data => data.id == id);
        const row = this.selectors.row(id);
        if (row === undefined || rowData === undefined) {
            return;
        }

        if (collapse) {
            this.collapsedIds.add(id);
        } else {
            this.collapsedIds.delete(id);
        }

        // Delete the child rows on collapse
        const children = rowData.children;
        if (collapse) {
            for (const childId of children) {
                const childNode = this.selectors.row(childId);
                childNode?.parentNode?.removeChild(childNode);
            }
            return;
        }

        // Create the child rows on expand
        const childRows = children
            .map(childId => this.data.find(data => data.id === childId))
            .filter(child => child !== undefined) as CtcRowData[];
        const rows = this.renderedRows(childRows)
            .map(Tables.createRow);
        row.after(...rows);

        // Propogate the checkmarks down to the newly created children
        const checkbox = this.selectors.check(id);
        if (checkbox) {
            for (const child of children) {
                const childCheckbox = this.selectors.check(child);
                if (!childCheckbox) {
                    continue;
                }

                childCheckbox.checked = checkbox.checked;
            }
        }
    }

    private toggleChildrenCheck(id: string, check: boolean) {
        const rowData = this.data.find(data => data.id == id);
        const row = this.selectors.row(id);
        if (row === undefined || rowData === undefined) {
            return;
        }

        const children = rowData.children;
        for (const childId of children) {
            const childCheck = this.selectors.check(childId);
            if (childCheck === undefined) {
                continue;
            }
            childCheck.checked = check;
        }
    }
}