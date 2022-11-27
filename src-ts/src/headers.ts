// import custom from "./custom.js";
// import exchange from "./exchange.js";

const selectors = {
    craftsHeader: document.querySelector('.header > div:nth-child(1)') as HTMLElement,
    customHeader: document.querySelector('.header > div:nth-child(2)') as HTMLElement,
    exchangeHeader: document.querySelector('.header > div:nth-child(3)') as HTMLElement,
    customContainer: document.querySelector('.container-custom') as HTMLElement,
    exchangeContainer: document.querySelector('.container-exchange') as HTMLElement,
}

export default {
    setupEvents() {
        selectors.customHeader.onclick = e => clickHeader(e);
        selectors.exchangeHeader.onclick = e => clickHeader(e);
    },
};

function clickHeader(event: MouseEvent) {
    const pairs = [
        { header: selectors.craftsHeader, container: null },
        { header: selectors.customHeader, container: selectors.customContainer },
        { header: selectors.exchangeHeader, container: selectors.exchangeContainer },
    ]

    for (const { header, container } of pairs) {
        header.removeAttribute('selected');
        container?.classList.add('collapsed');
    }

    (event.currentTarget as HTMLElement | null)?.setAttribute('selected', '');

    for (const { header, container } of pairs) {
        if (event.currentTarget === header) {
            container?.classList.remove('collapsed');
        }
    }
}
