export type ElemTagBaseOpts = { tag: string };
export type ElemChildBaseOpts = { children?: (HTMLElement|ElemAnyOpts)[] };
export type ElemInnerTextBaseOpts = { innerText?: string };
export type ElemValueBaseOpts = { value?: string };
export type ElemTypeBaseOpts = { type?: string };
export type ElemBaseOpts = { className?: string, id?: string };

export type ElemOptionOpts = ElemBaseOpts & ElemInnerTextBaseOpts & ElemValueBaseOpts;
export type ElemButtonOpts = ElemBaseOpts & ElemInnerTextBaseOpts & ElemValueBaseOpts;
export type ElemInputOpts = ElemBaseOpts & ElemTypeBaseOpts;
export type ElemDivOpts = ElemBaseOpts & ElemInnerTextBaseOpts & ElemChildBaseOpts;
export type ElemSpanOpts = ElemDivOpts;

export type ElemAnyOpts = ElemTagBaseOpts & ElemBaseOpts & ElemChildBaseOpts & ElemOptionOpts & ElemButtonOpts & ElemInputOpts & ElemDivOpts;

/////////////////////////////////////////////////////////

class Elem {
    static isHtmlElement(obj: any): boolean {
        return typeof obj === 'object' && obj.tagName !== undefined;
    }

    private static makeElem(inner: HTMLElement | ElemAnyOpts): HTMLElement {
        if (this.isHtmlElement(inner)) {
            return inner as HTMLElement;
        }

        inner = inner as ElemAnyOpts;
        if (inner.tag === undefined) {
            throw Error('Undefined element tag');
        }

        const elem = document.createElement(inner.tag);
        this.parseElem(elem, inner);
        return elem;
    }

    static makeTableRow(opts?: ElemChildBaseOpts): HTMLTableRowElement {
        return this.makeElem({ tag: 'tr', ...opts }) as HTMLTableRowElement;
    }

    static makeTableCell(opts?: ElemChildBaseOpts): HTMLTableCellElement {
        return this.makeElem({ tag: 'td', ...opts }) as HTMLTableCellElement;
    }

    static makeTableBody(opts?: ElemChildBaseOpts): HTMLElement {
        return this.makeElem({ tag: 'tbody', ...opts }) as HTMLElement;
    }

    static makeTableHeader(opts?: ElemChildBaseOpts): HTMLElement {
        return this.makeElem({ tag: 'thead', ...opts }) as HTMLElement;
    }

    static makeOption(opts?: ElemOptionOpts): HTMLOptionElement {
        return this.makeElem({ tag: 'option', ...opts }) as HTMLOptionElement;
    }

    private static makeInput(opts?: ElemInputOpts): HTMLInputElement {
        return this.makeElem({ tag: 'input', ...opts}) as HTMLInputElement;
    }

    static makeCheckbox(opts?: ElemInputOpts): HTMLInputElement {
        return this.makeInput({ type: 'checkbox', ...opts });
    }

    static makeButton(opts?: ElemButtonOpts): HTMLButtonElement {
        return this.makeElem({ tag: 'button', ...opts }) as HTMLButtonElement;
    }

    static makeDiv(opts?: ElemDivOpts): HTMLDivElement {
        return this.makeElem({ tag: 'div', ...opts }) as HTMLDivElement;
    }

    static makeSpan(opts?: ElemSpanOpts): HTMLSpanElement {
        return this.makeElem({ tag: 'span', ...opts }) as HTMLDivElement;
    }

    private static parseElem(elem: any, inner: ElemAnyOpts): void {
        for (const [ key, value ] of Object.entries(inner)) {
            if (key === 'children') {
                for (const child of value) {
                    elem.appendChild(this.makeElem(child as ElemAnyOpts));
                }

                continue;
            }

            if (key === 'tag') {
                continue;
            }

            elem[key] = value;
        }
    }
}

export default Elem;
