var craftInfo = (() => {
    this.groups = null;
    this.curGroup = null;

    const NUM_COLS = 3;

    this.fetchData = async () => {
        craftInfo.groups = await (await util.fetchFile("crafts.json")).json();
    }

    this.createTables = () => {
        if (!craftInfo.groups) {
            console.error("JSON data has not been fetched");
            return;
        }

        const _craftGroupsTable = document.getElementById("craft-group-list");

        for (const group of craftInfo.groups) {
            const div = util.makeElem({ tag: 'div', className: 'table-name', innerText: group.name });
            div.onclick = _ => _showCraftTable(group);
            _craftGroupsTable.appendChild(div);
        }
    }

    this.setupEvents = () => {
        const _arr = [
            document.getElementById("craft-min-profit"),
            document.getElementById("craft-min-velocity"),
        ];

        for (const _element of _arr) {
            _element.addEventListener('input', _ => {
                _showCraftTable(null)
            });
        }
    }

    const _createCraftTable = group => {
        const _columns = ["Name", "Velocity NQ", "Velocity HQ", "Local Sell Price", "Best Buy Price", "Profit"];

        const _name = v => _td => { _td.className = "item-name"; _td.appendChild(document.createTextNode(v.toString())); };
        const _profit = v => (_td, _tr) => {
            _tr.className = (v > 0) ? "profit-positive" : "profit-negative";
            _td.appendChild(document.createTextNode(v.toString()));
        };
        const _velocity = v => _td => {
            const _div = document.createElement("div");
            _div.className = "velocity";
            const _velocity = document.createElement("div");
            const _count = document.createElement("div");
            _velocity.appendChild(document.createTextNode(v.velocity.toFixed(1).toString()));
            _count.appendChild(document.createTextNode(v.count.toString()));

            _div.appendChild(_velocity);
            _div.appendChild(_count);
            _td.appendChild(_div);
        };

        const _craftGroup = _getFilteredCraftGroup(group);
        const _isAllNq = _craftGroup.every(craft => craft.velocity_hq.velocity == 0);
        const _rows = _craftGroup.map(craft => {
            return [
                _name(craft.item_name),
                _velocity(craft.velocity_nq),
                _isAllNq ? "" : _velocity(craft.velocity_hq),
                craft.local_sell_price,
                craft.best_buy_price,
                _profit(craft.profit)
            ];
        });
    
        const _table = tables.createTable(_columns, _rows);
        return _table;
    }
    
    const _showCraftTable = group => {
        if (group == craftInfo.curGroup) {
            return;
        }
    
        if (group != null) {
            craftInfo.curGroup = group;
        } else {
            group = craftInfo.curGroup;
        }
    
        const _craftTableDiv = document.getElementById("craft-table-div");
        const _existingTable = document.getElementById("craft-table-cur");
        if (_existingTable) {
            _craftTableDiv.removeChild(_existingTable);
        }
    
        const _table = _createCraftTable(group);
        _table.id = "craft-table-cur";
    
        _craftTableDiv.appendChild(_table);
    }
    
    const _getCraftFilters = () => {
        var _minProfit = 0;
        try {
            _minProfit = parseInt(document.getElementById("craft-min-profit").value);
        } catch(e) {
            console.error(e);
        }
    
        var _minVelocity = 3.0;
        try {
            _minVelocity = parseFloat(document.getElementById("craft-min-velocity").value);
        } catch(e) {
            console.error(e);
        }
    
        return {
            minProfit: _minProfit,
            minVelocity: _minVelocity,
        };
    }
    
    const _getFilteredCraftGroup = group => {
        const _filters = _getCraftFilters();
        const _rows = group.crafts;
        _rows.sort((a, b) => b.profit - a.profit);
    
        return _rows.filter(craft => {
            if (craft.profit < _filters.minProfit) return false;
            if (craft.velocity_nq.velocity + craft.velocity_hq.velocity < _filters.minVelocity) return false;
            return true;
        });
    }

    return this;
})();

