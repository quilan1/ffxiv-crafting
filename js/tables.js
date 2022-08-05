const tables = {};
const tablesProto = Object.getPrototypeOf(tables);

tablesProto.createTable = (columns, rows) => {
    var _table = document.createElement("table");
    var _thead = _createTableHeader(columns);
    var _tbody = _createTableRows(rows);
    if (_thead) _table.appendChild(_thead);
    _table.appendChild(_tbody);
    return _table;
}

_createTableHeader = (columns) => {
    if (!columns) {
        return null;
    }

    var _thead = document.createElement("thead");
    var _tr = document.createElement("tr");
    for (const column of columns) {
        const _th = document.createElement("th");
        const _text = document.createTextNode(column);
        _th.appendChild(_text);
        _tr.appendChild(_th);
    }
    _thead.appendChild(_tr);
    return _thead;
}

_createTableRows = (rows) => {
    var _tbody = document.createElement("tbody");
    for (const row of rows) {
        const _tr = document.createElement("tr");
        for (const data of row) {
            const _td = document.createElement("td");
            if (typeof data === "string" || typeof data === "number") {
                const _text = document.createTextNode(data.toString());
                _td.appendChild(_text);
            } else if (typeof data === "function") {
                data(_td, _tr);
            } else {
                _td.appendChild(data);
            }
            _tr.appendChild(_td);
        }
        _tbody.appendChild(_tr);
    }
    return _tbody;
}
