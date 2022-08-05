var customInfo = (() => {
    let selectors;

    const _getSelectors = () => {
        const makeData = (parent, child) => {
            return {
                parent: document.querySelector(parent),
                cur: () => document.querySelector(child),
                set(_new) {
                    util.removeChild(this.parent, this.cur());
                    this.parent.appendChild(_new);
                }
            }
        };

        return {
            filters: {
                search: document.querySelector('#custom-filters'),
                list: document.querySelector('#custom-filters-list'),
                new: document.querySelector('#custom-filters-new'),
                save: document.querySelector('#custom-filters-save'),
                delete: document.querySelector('#custom-filters-delete'),
                refresh: document.querySelector('#custom-filters-refresh'),
            },
            data: {
                table: makeData('#custom-table-div', '#custom-table-cur'),
                world: makeData('#custom-world-div', '#custom-world-cur'),
            },
            row: id => document.querySelector(`table#custom-table-cur > tbody > tr:nth-child(${Number(id)+1})`),
            rowCheck: id => selectors.row(id).querySelector(`:scope > td:nth-child(1) > input[type='checkbox']`),
            rowPlus: id => selectors.row(id).querySelector(`:scope > td:nth-child(2) > button`),
        };
    };

    const _regexNameSkip = /(Fire|Ice|Wind|Earth|Lightning|Water) (Shard|Crystal|Cluster)/;

    this.setupEvents = () => {
        selectors = _getSelectors();
        selectors.filters.refresh.onclick = _ => _refreshCustom();
    };

    const _refreshCustom = async () => {
        const analysisInfo = await _enumerateAnalyses();
        const rows = Object.values(analysisInfo).map(analysisData => _createTableRow(analysisData));

        const columns = ["☑", "-", "Name", "Best Price", "Best Child Price", "Worlds"];
        const table = tables.createTable(columns, rows);
        table.id = "custom-table-cur";
        selectors.data.table.set(table);

        for (const analysisData of Object.values(analysisInfo)) {
            _processRow(analysisData, analysisInfo);
        }

        for (const analysisData of Object.values(analysisInfo)) {
            const { id, analysis: { buy_price, best_buy_price } } = analysisData;
            if (buy_price > 0 && buy_price < best_buy_price) {
                selectors.rowPlus(id).onclick();
            }
        }

        if (Object.keys(analysisInfo).length > 0) {
            let check = selectors.rowCheck(0);
            check.checked = true;
            check.onclick();
        }
    };

    const _enumerateAnalyses = async () => {
        const encFilters = encodeURIComponent(selectors.filters.search.value);
        selectors.filters.refresh.disabled = true;
        let analyses;
        try {
            let request = await util.fetch(`v1/custom-filter?filters=${encFilters}`);
            analyses = await request.json();
        } catch(err) {
            console.error(err);
            return;
        } finally {
            selectors.filters.refresh.disabled = false;
        }

        if (analyses.length > 1) {
            analyses = {
                name: "All Crafts",
                count: 1,
                buy_price: "",
                best_buy_price: "",
                buy_worlds: [],
                child_analyses: analyses,
            };
        } else if (analyses.length == 1) {
            analyses = analyses[0];
        } else {
            return {};
        }

        const _assignId = (data, id, analysis, depth) => {
            const info = {
                id: id[0],
                depth: depth,
                children: [],
                analysis: analysis,
            };

            data[info.id] = info;
            id[0] += 1;
            for (const childAnalysis of analysis.child_analyses) {
                if (_regexNameSkip.test(childAnalysis.name)) {
                    continue;
                }

                const child = _assignId(data, id, childAnalysis, depth + 1);
                info.children.push(child.id);
            }

            return info;
        }

        const data = {};
        _assignId(data, [0], analyses, 0);
        return data;
    };

    const _createTableRow = (analysisInfo) => {
        const { depth, analysis } = analysisInfo;

        const _name = v => td => {
            td.className = "item-name";
            td.style.paddingLeft = `${20*depth}px`;
            td.appendChild(document.createTextNode(v.toString()));
        };

        const _check = td => {
            const check = document.createElement("input");
            check.type = "checkbox";
            td.appendChild(check);
        };

        const _worlds = worlds => td => {
            if (worlds.length == 1 && worlds[0].count == 0) {
                return;
            }

            const worldsDiv = document.createElement('div');
            worldsDiv.classList.add('world-purchase-list');
            for (const worldInfo of worlds) {
                const worldInfoDiv = document.createElement('div');
                worldInfoDiv.classList.add('world-info');

                const countDiv = document.createElement('div');
                const worldDiv = document.createElement('div');
                const priceDiv = document.createElement('div');
                countDiv.innerText = `${worldInfo.count}x`;
                worldDiv.innerText = worldInfo.world;
                priceDiv.innerText = worldInfo.price;

                worldInfoDiv.appendChild(countDiv);
                worldInfoDiv.appendChild(worldDiv);
                worldInfoDiv.appendChild(priceDiv);

                worldsDiv.appendChild(worldInfoDiv);
            }

            td.appendChild(worldsDiv);
        }

        return [
            _check,
            "",
            _name(`${analysis.count}x ${analysis.name}`),
            analysis.buy_price,
            analysis.best_buy_price,
            _worlds(analysis.buy_worlds),
        ];
    };

    const _processRow = (analysisData, analysisInfo) => {
        analysisData.isCollapsed = false;

        const check = selectors.rowCheck(analysisData.id);
        check.onclick = _ => {
            _checkChildren(check.checked, analysisData.id, analysisInfo);
            _displayWorldInfo(analysisInfo);
        }

        if (analysisData.children && analysisData.children.length > 0) {
            const plus = document.createElement('button');
            plus.innerText = '-';
            plus.classList.add('collapsable');
            plus.onclick = _ => {
                plus.innerText = ['-', '+'][Number(plus.innerText == '-')];
                _toggleRowChildren(analysisData.id, analysisInfo);
                _displayWorldInfo(analysisInfo);
            }

            selectors.row(analysisData.id).cells[1].appendChild(plus);
        }
    }

    const _toggleRowChildren = (id, analysisInfo) => {
        const _collapseChildren = id => {
            const { children } = analysisInfo[id];
            selectors.row(id).classList.add("collapsed");
            for (const childId of children) {
                _collapseChildren(childId);
            }
        };

        const _expandChildren = id => {
            const { children, isCollapsed } = analysisInfo[id];
            selectors.row(id).classList.remove("collapsed");
            if (!isCollapsed) {
                for (const childId of children) {
                    _expandChildren(childId);
                }
            }
        };

        for(const childId of analysisInfo[id].children) {
            if (analysisInfo[id].isCollapsed) {
                _expandChildren(childId);
            } else {
                _collapseChildren(childId);
            }
        }

        analysisInfo[id].isCollapsed ^= true;
    }

    const _checkChildren = (checked, id, analysisInfo) => {
        selectors.rowCheck(id).checked = checked;
        for (const childId of analysisInfo[id].children) {
            _checkChildren(checked, childId, analysisInfo);
        }
    }

    const _displayWorldInfo = (analysisInfo) => {
        const worldDiv = document.createElement('div');
        worldDiv.id = 'custom-world-cur';

        const worlds = {};
        for (const analysisData of Object.values(analysisInfo)) {
            const { id, analysis, isCollapsed, children } = analysisData;
            const { name, buy_worlds } = analysis;

            const check = selectors.rowCheck(id);
            if (!check.checked || selectors.row(id).classList.contains('collapsed')) {
                continue;
            }

            if (!isCollapsed && children.length > 0) {
                continue;
            }

            for (const { count, world, price } of buy_worlds) {
                if (!worlds[world]) {
                    worlds[world] = [];
                }

                worlds[world].push([name, count, price]);
            }
        }

        for (const [ world, worldInfo ] of util.sorted(Object.entries(worlds))) {
            const childrenDivs = worldInfo.map( ([ name, count, price ]) => {
                return util.makeDiv([
                    util.makeDiv(`${count}x`),
                    util.makeDiv(`[<= ${price} gil]`),
                    util.makeDiv(name),
                ]);
            });

            const titleDiv = util.makeDiv(world);
            const childrenDiv = util.makeDiv(childrenDivs);
            const curWorldDiv = util.makeDiv([titleDiv, childrenDiv]);
            worldDiv.appendChild(curWorldDiv);
        }

        selectors.data.world.set(worldDiv);
    }

    return this;
})();
