var util = {
    fetchFile: (fileName) => {
        return util.fetch(`js/${fileName}`);
    },

    fetch: (path, args) => {
        return fetch(`http://127.0.0.1:3001/${path}`, args);
    },

    makeDiv: inner => {
        const div = document.createElement('div');
        if (inner !== undefined && (typeof inner === 'string' || typeof inner === 'number')) {
            div.innerText = inner;
        } else if (Array.isArray(inner)) {
            for (const elem of inner) {
                div.appendChild(elem);
            }
        } else {
            throw Error('Invalid makeDiv inner type');
        }
        return div;
    },

    sorted: _arr => {
        const arr = [..._arr];
        arr.sort();
        return arr;
    },

    removeChild: (parent, child) => {
        if (child) {
            parent.removeChild(child);
        }
    }
};
