const util = (() => {
    const _fetch = fetch;

    this.fetchFile = (fileName) => {
        return util.fetch(`js/${fileName}`);
    };

    this.fetch = (path, args) => {
        return _fetch(`http://127.0.0.1:3001/${path}`, args);
    };

    this.makeElem = inner => {
        if (typeof inner === 'object' && inner.tagName !== undefined) {
            return inner;
        }

        if (inner.tag === undefined) {
            throw Error('Undefined element tag');
        }

        const elem = document.createElement(inner.tag);
        _parseElem(elem, inner);
        return elem;
    }

    const _parseElem = (elem, inner) => {
        for (const [ key, value ] of Object.entries(inner)) {
            if (key === 'children') {
                for (const child of inner.children) {
                    elem.appendChild(util.makeElem(child));
                }

                continue;
            }

            if (key === 'tag') {
                continue;
            }

            elem[key] = value;
        }

        // if (inner.id != null) {
        //     elem.id = inner.id;
        // }
        // if (inner.innerText != null) {
        //     elem.innerText = inner.innerText;
        // }
        // if (inner.text != null) {
        //     elem.text = inner.text;
        // }
        // if (inner.value != null) {
        //     elem.value = inner.value;
        // }
        // if (inner.children != null) {
        // }
    }

    this.sorted = _arr => {
        const arr = [..._arr];
        arr.sort();
        return arr;
    };

    this.removeChild = (parent, child) => {
        if (child) {
            parent.removeChild(child);
        }
    };

    return this;
})();
