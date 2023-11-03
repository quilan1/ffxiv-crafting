/* eslint-disable @typescript-eslint/no-explicit-any, @typescript-eslint/no-unnecessary-condition */
/* eslint-disable @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-return */

const knownObjects = new Map();
const generate = function* () {
    while (true) {
        const random = Math.random()
            .toString(16)
            .slice(2, 8);
        yield `0x${random}`;
    }
};
const generator = generate();

const refs = {};
export const findRef = (obj: any) => {
    let address;
    if (knownObjects.has(obj)) {
        address = knownObjects.get(obj);
    } else {
        address = generator.next().value;
        knownObjects.set(obj, address);
        (refs as any)[address] = obj;
    }
    return address;
};
