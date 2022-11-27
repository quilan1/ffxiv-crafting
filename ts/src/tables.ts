import Util from './util.js';
import Elem from './elem.js';

export type TableRowFunction = (td: HTMLTableCellElement, tr: HTMLTableRowElement) => void;
export type TableRowElement = string | number | TableRowFunction | HTMLElement;
export type TableRow = TableRowElement[];

class Tables {
    static createTable(columns: string[], rows: TableRow[]): HTMLElement {
        var _table = document.createElement("table");
        var _thead = this.createTableHeader(columns);
        var _tbody = this.createRows(rows);
        if (_thead) _table.appendChild(_thead);
        _table.appendChild(_tbody);
        return _table;
    }

    static createRows(rows: TableRow[]): HTMLElement {
        return Elem.makeTableBody({ children: rows.map(row => this.createRow(row))});
    }

    static createRow(row: TableRow): HTMLElement {
        const tr = Elem.makeTableRow();
        for (const data of row) {
            const td = Elem.makeTableCell();
            if (typeof data === "string" || typeof data === "number") {
                td.innerText = data.toString();
            } else if (typeof data === "function") {
                data(td, tr);
            } else {
                td.appendChild(data);
            }
            tr.appendChild(td);
        }

        return tr;
    }

    private static createTableHeader(columns?: string[]): HTMLElement | null {
        if (!columns) {
            return null;
        }

        return Elem.makeTableHeader({
            children: [{
                tag: 'tr',
                children: columns.map(column => ({ tag: 'th', innerText: column }))
            }]
        });
    };

}

export default Tables;
