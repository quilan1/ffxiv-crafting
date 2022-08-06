const customInfo = (() => {
    let selectors;
    const savedFilters = [];

    const _getSelectors = () => {
        const makeData = (parent, child) => {
            return {
                cur: () => document.querySelector(child),
                tbody: () => document.querySelector(`${child} > tbody`),
                set(_new) {
                    const me = this.cur();
                    me.parentNode.replaceChild(_new, me);
                },
                setBody(_new) {
                    const me = this.tbody();
                    me.parentNode.replaceChild(_new, me);
                },
            }
        };

        return {
            filters: {
                search: document.querySelector('#custom-filters'),
                list: document.querySelector('#custom-filters-list'),
                load: document.querySelector('#custom-filters-load'),
                save: document.querySelector('#custom-filters-save'),
                saveAs: document.querySelector('#custom-filters-save-as'),
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
        selectors.filters.refresh.onclick = _ => _refresh.refresh();
        selectors.filters.load.onclick = _ => _filters.load();
        selectors.filters.save.onclick = _ => _filters.save();
        selectors.filters.saveAs.onclick = _ => _filters.saveAs();
        selectors.filters.delete.onclick = _ => _filters.delete();
        selectors.filters.list.onchange = _ => _filters.changeSelection();
        _filters.loadFromStorage();
        _filters.changeSelection();
    };

    const _refresh = (() => {
        this.refresh = async () => {
            const analysisInfo = await _enumerateAnalyses();
            const rows = Object.values(analysisInfo).map(analysisData => _createTableRow(analysisData));
            selectors.data.table.setBody(tables.createRows(rows));

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
                td.appendChild(util.makeElem({ tag: 'input', type: 'checkbox' }));
            };

            const _worlds = worlds => td => {
                if (worlds.length == 1 && worlds[0].count == 0) {
                    return;
                }

                const worldsDiv = util.makeElem({
                    tag: 'div',
                    className: 'world-purchase-list',
                    children: worlds.map(worldInfo => ({
                        tag: 'div',
                        className: 'world-info',
                        children: [
                            { tag: 'div', innerText: `${worldInfo.count}x` },
                            { tag: 'div', innerText: worldInfo.world },
                            { tag: 'div', innerText: worldInfo.price },
                        ]
                    })),
                });

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
                const plus = util.makeElem({ tag: 'button', innerText: '-', className: 'collapsable' });
                plus.onclick = _ => {
                    plus.innerText = ['-', '+'][Number(plus.innerText == '-')];
                    _toggleRowChildren(analysisData.id, analysisInfo);
                    _displayWorldInfo(analysisInfo);
                }

                selectors.row(analysisData.id).cells[1].appendChild(plus);
            }
        };

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
        };

        const _checkChildren = (checked, id, analysisInfo) => {
            selectors.rowCheck(id).checked = checked;
            for (const childId of analysisInfo[id].children) {
                _checkChildren(checked, childId, analysisInfo);
            }
        };

        const _displayWorldInfo = (analysisInfo) => {
            const worldDiv = util.makeElem({ tag: 'div', id: 'custom-world-cur' });

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
                    return {
                        tag: 'div',
                        children: [
                            { tag: 'div', innerText: `${count}x` },
                            { tag: 'div', innerText: `[<= ${price} gil]` },
                            { tag: 'div', innerText: name },
                        ]
                    };
                });

                const titleDiv = util.makeElem({ tag: 'div', innerText: world });
                const childrenDiv = util.makeElem({ tag: 'div', children: childrenDivs });
                const curWorldDiv = util.makeElem({ tag: 'div', children: [titleDiv, childrenDiv] });
                worldDiv.appendChild(curWorldDiv);
            }

            selectors.data.world.set(worldDiv);
        };

        return this;
    })();

    const _filters = (() => {
        this.save = () => {
            const list = selectors.filters.list;
            if (list.selectedIndex == 0) {
                this.changeSelection();
                return;
            }

            const value = selectors.filters.search.value;
            savedFilters[list.selectedIndex-1].value = value;
            list.options[list.selectedIndex].value = value;
        }

        this.saveAs = () => {
            const value = selectors.filters.search.value;
            const saveName = prompt('With what name would you like to save this filter as?', value);
            if (saveName === null) {
                return;
            }

            savedFilters.push({ name: saveName, filter: value });
            _addOption(saveName, value);

            const list = selectors.filters.list;
            list.selectedIndex = list.options.length - 1;
            this.changeSelection();
        };

        _addOption = (innerText, value) => {
            const option = util.makeElem({ tag: 'option', innerText: innerText, value: value });
            const list = selectors.filters.list;
            list.add(option);
            return option;
        }

        this.load = () => {
            const curSelected = selectors.filters.list.value;
            if (!curSelected) {
                this.changeSelection();
                return;
            }

            selectors.filters.search.value = curSelected;
            this.changeSelection();
        };

        this.delete = () => {
            const list = selectors.filters.list;
            if (list.selectedIndex == 0) {
                this.changeSelection();
                return;
            }

            savedFilters.splice(list.selectedIndex-1, 1);
            list.options.remove(list.selectedIndex);
            this.changeSelection();
        }

        this.loadFromStorage = () => {
            const storageFilters = localStorage.getItem('custom-filters');
            if (storageFilters) {
                savedFilters.splice(0, savedFilters.length);
                savedFilters.push(...JSON.parse(storageFilters));
                for (const { name, filter } of savedFilters) {
                    _addOption(name, filter);
                }
            }
        }

        this.changeSelection = () => {
            if (selectors.filters.list.selectedIndex == 0) {
                selectors.filters.save.disabled = true;
                selectors.filters.delete.disabled = true;
            } else {
                selectors.filters.save.disabled = false;
                selectors.filters.delete.disabled = false;
            }

            localStorage.setItem('custom-filters', JSON.stringify(savedFilters));
        }

        return this;
    })();

    return this;
})();
