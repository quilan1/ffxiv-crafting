const tables = (() => {
    this.createTable = (columns, rows) => {
        var _table = document.createElement("table");
        var _thead = _createTableHeader(columns);
        var _tbody = this.createRows(rows);
        if (_thead) _table.appendChild(_thead);
        _table.appendChild(_tbody);
        return _table;
    }

    this.createRows = rows => {
        return util.makeElem({ tag: 'tbody', children: rows.map(row => this.createRow(row))});
    }

    this.createRow = row => {
        const tr = util.makeElem({ tag: 'tr' });
        for (const data of row) {
            const td = util.makeElem({ tag: 'td' });
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

    _createTableHeader = (columns) => {
        if (!columns) {
            return null;
        }

        return util.makeElem({
            tag: 'thead',
            children: [{
                tag: 'tr',
                children: columns.map(column => ({ tag: 'th', innerText: column }))
            }]
        });
    }

    return this;
})();