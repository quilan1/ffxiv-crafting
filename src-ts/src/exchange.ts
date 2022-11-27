import RecStatistics from "./custom/rec_statistics.js";
import CustomInfo from "./custom/custom_info.js";
import Elem, { ElemAnyOpts } from "./elem.js";

const exchangeCosts = [
    { type: 'purpleScrips', name: "Purple Crafting Scrips", search: ":name ^Rarefied, :rlevel 90, :count 20", exchange: 20*144 },
    { type: 'whiteScrips', name: "White Crafting Scrips", search: ":name ^Rarefied, :rlevel 61|69, :count 20", exchange: 20*54 },
    { type: 'tickets', name: "Skybuilders' Scrips", search: ":name ^Grade 4 Skybuilders', :rlevel 80, :count 100", exchange: 100*63 },
];

const exchangeProfits = [
    { purpleScrips: 125, name: 'Immutable Solution' },
    { purpleScrips: 500, name: "Craftsman's Competence Materia X" },
    { purpleScrips: 500, name: "Craftsman's Cunning Materia X" },
    { purpleScrips: 500, name: "Craftsman's Command Materia X" },

    { whiteScrips: 250, name: "Craftsman's Competence Materia IX" },
    { whiteScrips: 250, name: "Craftsman's Cunning Materia IX" },
    { whiteScrips: 250, name: "Craftsman's Command Materia IX" },
    { whiteScrips: 200, name: "Craftsman's Competence Materia VII" },
    { whiteScrips: 200, name: "Craftsman's Cunning Materia VII" },
    { whiteScrips: 200, name: "Craftsman's Command Materia VII" },
    { whiteScrips: 25, name: "Craftsman's Competence Materia IV" },
    { whiteScrips: 25, name: "Craftsman's Cunning Materia IV" },
    { whiteScrips: 25, name: "Craftsman's Command Materia IV" },

    { tickets: 200, name: 'Skysteel Ingot' },
    { tickets: 200, name: 'Skysteel Cloth' },
    { tickets: 200, name: 'Skysteel Leather' },
    { tickets: 240, name: "Gatherer's Guerdon Materia VII" },
    { tickets: 300, name: "Gatherer's Guerdon Materia VIII" },
    { tickets: 100, name: "Ruby Red Dye" },
    { tickets: 100, name: "Cherry Pink Dye" },
    { tickets: 100, name: "Canary Yellow Dye" },
    { tickets: 100, name: "Vanilla Yellow Dye" },
    { tickets: 100, name: "Dragoon Blue Dye" },
    { tickets: 100, name: "Turquoise Blue Dye" },
    { tickets: 500, name: "Gunmetal Black Dye" },
    { tickets: 500, name: "Pearl White Dye" },
    { tickets: 500, name: "Metallic Brass Dye" },
    { tickets: 8400, name: "Albino Karakul Horn" },
    { tickets: 8400, name: "Ufiti Horn" },
    { tickets: 8400, name: "Megalotragus Horn" },
    { tickets: 8400, name: "Big Shell Whistle" },
    { tickets: 8400, name: "Antelope Doe Horn" },
    { tickets: 1800, name: "Ballroom Etiquette - The Winsome Wallflower" },
    { tickets: 1800, name: "Ballroom Etiquette - Intelligent Impressions" },
    { tickets: 1800, name: "Ballroom Etiquette - Emphatic Elucidation" },
    { tickets: 1800, name: "Ballroom Etiquette - Uncouth Congratulations" },
    // { tickets: 1800, name: "Ballroom Etiquette - Next, Godliness" },
    { tickets: 1800, name: "Ballroom Etiquette - Concealing Meals" },
    { tickets: 900, name: "Ballroom Etiquette - Well Bread" },
    { tickets: 1800, name: "Modern Aesthetics - Modern Legend" },
    { tickets: 1800, name: "Modern Aesthetics - Controlled Chaos" },
    { tickets: 1800, name: "Modern Aesthetics - Saintly Style" },
    { tickets: 1200, name: "Miniature White Knight" },
    { tickets: 1200, name: "Cerberpup" },
    { tickets: 800, name: "Paissa Brat" },
    { tickets: 800, name: "Hunting Hawk" },
    { tickets: 800, name: "Baby Brachiosaur" },
    // { tickets: 800, name: "Pegasus Colt" },
    { tickets: 1200, name: "Machinist Barding" },
    { tickets: 1200, name: "Safety in Numbers Orchestrion Roll" },
    { tickets: 1200, name: "The Mendicant's Relish Orchestrion Roll" },
    { tickets: 1200, name: "The Heavens' Ward Orchestrion Roll" },
    { tickets: 1200, name: "Hearthward Orchestrion Roll" },
    { tickets: 1200, name: "What Is Love? Orchestrion Roll" },
    { tickets: 1200, name: "Skyrise Orchestrion Roll" },
    { tickets: 600, name: "Jewel Orchestrion Roll" },
    { tickets: 600, name: "Paradise Found Orchestrion Roll" },
    { tickets: 600, name: "Fealty Orchestrion Roll" },
    { tickets: 600, name: "Stone and Steel Orchestrion Roll" },
    { tickets: 600, name: "Order Yet Undeciphered Orchestrion Roll" },
    { tickets: 600, name: "Freefall Orchestrion Roll" },
    { tickets: 1800, name: "Parasol" },
    { tickets: 1800, name: "Cheerful Checkered Parasol" },
    { tickets: 1800, name: "Pastoral Dot Parasol" },
    { tickets: 900, name: "Sky Blue Parasol" },
    { tickets: 900, name: "Calming Checkered Parasol" },
    { tickets: 2200, name: "Craftsman's Coverall Top" },
    { tickets: 2200, name: "Craftsman's Singlet" },
    { tickets: 2200, name: "Craftsman's Apron" },
    { tickets: 2000, name: "Craftsman's Coverall Bottoms" },
    { tickets: 2000, name: "Craftsman's Leather Trousers" },
    { tickets: 1200, name: "Craftsman's Leather Shoes" },
    { tickets: 1200, name: "Skyworker's Helmet" },
    { tickets: 2200, name: "Skyworker's Singlet" },
    { tickets: 1200, name: "Skyworker's Gloves" },
    { tickets: 2200, name: "Skyworker's Bottoms" },
    { tickets: 1200, name: "Skyworker's Boots" },
]

const selectors = {
    refresh: document.querySelector('#exchange-refresh') as HTMLButtonElement,
    cur: () => document.querySelector('#exchange-cur') as HTMLElement,
    status: document.querySelector('#exchange-refresh-status') as HTMLElement,
}

export default {
    setupEvents() {
        selectors.refresh.onclick = _ => this._calculate();
    },

    _pricePromise(search: string): Promise<CustomInfo> {
        return CustomInfo.fetch(search);
    },

    _profitPromise(type: string): Promise<CustomInfo> {
        const purchases = exchangeProfits.filter(item => (item as any)[type] !== undefined);
        const search = ":name (" + purchases.map(item => `^${item.name}\$`).join("|") + ")";
        return CustomInfo.fetch(search);
    },

    async _calculate() {
        try {
            selectors.refresh.disabled = true;
            const newDiv = Elem.makeDiv({ id: 'exchange-cur' });

            let exchangeCostInfo = [];
            for (const { name, type, search, exchange } of exchangeCosts) {
                const pricePromise = this._pricePromise(search);
                const profitPromise = this._profitPromise(type);
                exchangeCostInfo.push({ name, type, exchange, pricePromise, profitPromise });
            }

            const start = Date.now();
            for (const { name, type, exchange, pricePromise, profitPromise } of exchangeCostInfo) {
                selectors.status.innerText = `Fetching ${name} prices`;
                const result = await this._calculatePrice(type, pricePromise, exchange);
                selectors.status.innerText = `Fetching ${name} exchange items`;
                const profitResults = await this._calculateProfits(type, profitPromise, result.pricePer) as unknown as ElemAnyOpts[];
                newDiv.appendChild(Elem.makeDiv({ innerText: name }));
                newDiv.appendChild(Elem.makeDiv({ className: 'exchange-profit-list', children: profitResults }));
            }
            selectors.status.innerText = '';
            selectors.status.innerText = `Time taken: ${Date.now()-start}`;
            selectors.cur().parentNode?.replaceChild(newDiv, selectors.cur());
        } finally {
            selectors.refresh.disabled = false;
        }
    },

    async _calculatePrice(type: string, pricePromise: Promise<CustomInfo>, exchange: number) {
        const results = await pricePromise;
        const cheapestList = results.rec_statistics.values;
        cheapestList.sort((a, b) => a.buyCraftPrice - b.buyCraftPrice);
        const stats = cheapestList[0];

        return {
            type: type,
            name: stats.item.name,
            pricePer: stats.buyCraftPrice / exchange,
        };
    },

    async _calculateProfits(type: string, profitPromise: Promise<CustomInfo>, pricePer: number) {
        const purchases = exchangeProfits.filter(item => (item as any)[type] !== undefined);
        const searchResults = await profitPromise;

        const results = [];
        for (const purchase of purchases) {
            const stats = searchResults.rec_statistics.values.find(stats => stats.item.name === purchase.name);
            if (stats === undefined) {
                console.error(`${purchase.name} missing!`);
                continue;
            }

            const itemPricePer = (purchase as any)[type] * pricePer;
            const sellPrice = Math.max(stats.buyCraftPrice, stats.medSellPrice ?? Number.MIN_SAFE_INTEGER);
            results.push({
                tag: 'div',
                className: 'exchange-profit',
                children: [
                    { tag: 'div', innerText: sellPrice },
                    { tag: 'div', innerText: Math.round(itemPricePer) },
                    { tag: 'div', innerText: Math.round(10*sellPrice/itemPricePer)/10 },
                    { tag: 'div', innerText: `${purchase.name} [${stats.item.statistics.homeworldVelocity?.aq?.toFixed(2) ?? '--'}]` },
                ]
            });
        }

        results.sort((a, b) => (a.children[2].innerText as number) - (b.children[2].innerText as number));

        return results;
    }
};