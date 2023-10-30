export const dataCenterOf = (world: string): string => {
    const dataCenterInfo = allDataCenters.find(info => info.world === world);
    if (dataCenterInfo === undefined)
        throw new Error(`Invalid world for data center: {world}`);
    return dataCenterInfo.dataCenter;
}

interface DataCenterInfo {
    region: string,
    dataCenter: string,
    world: string,
}

export const allDataCenters: DataCenterInfo[] = [
    { region: 'North America', dataCenter: 'Aether', world: 'Adamantoise' },
    { region: 'North America', dataCenter: 'Aether', world: 'Cactuar' },
    { region: 'North America', dataCenter: 'Aether', world: 'Faerie' },
    { region: 'North America', dataCenter: 'Aether', world: 'Gilgamesh' },
    { region: 'North America', dataCenter: 'Aether', world: 'Jenova' },
    { region: 'North America', dataCenter: 'Aether', world: 'Midgardsormr' },
    { region: 'North America', dataCenter: 'Aether', world: 'Sargatanas' },
    { region: 'North America', dataCenter: 'Aether', world: 'Siren' },

    { region: 'North America', dataCenter: 'Crystal', world: 'Balmung' },
    { region: 'North America', dataCenter: 'Crystal', world: 'Brynhildr' },
    { region: 'North America', dataCenter: 'Crystal', world: 'Coeurl' },
    { region: 'North America', dataCenter: 'Crystal', world: 'Diabolos' },
    { region: 'North America', dataCenter: 'Crystal', world: 'Goblin' },
    { region: 'North America', dataCenter: 'Crystal', world: 'Malboro' },
    { region: 'North America', dataCenter: 'Crystal', world: 'Mateus' },
    { region: 'North America', dataCenter: 'Crystal', world: 'Zalera' },

    { region: 'North America', dataCenter: 'Primal', world: 'Behemoth' },
    { region: 'North America', dataCenter: 'Primal', world: 'Excalibur' },
    { region: 'North America', dataCenter: 'Primal', world: 'Exodus' },
    { region: 'North America', dataCenter: 'Primal', world: 'Famfrit' },
    { region: 'North America', dataCenter: 'Primal', world: 'Hyperion' },
    { region: 'North America', dataCenter: 'Primal', world: 'Lamia' },
    { region: 'North America', dataCenter: 'Primal', world: 'Leviathan' },
    { region: 'North America', dataCenter: 'Primal', world: 'Ultros' },

    { region: 'North America', dataCenter: 'Dynamis', world: 'Halicarnassus' },
    { region: 'North America', dataCenter: 'Dynamis', world: 'Maduin' },
    { region: 'North America', dataCenter: 'Dynamis', world: 'Marilith' },
    { region: 'North America', dataCenter: 'Dynamis', world: 'Seraph' },

    { region: 'Europe', dataCenter: 'Chaos', world: 'Cerberus' },
    { region: 'Europe', dataCenter: 'Chaos', world: 'Louisoix' },
    { region: 'Europe', dataCenter: 'Chaos', world: 'Moogle' },
    { region: 'Europe', dataCenter: 'Chaos', world: 'Omega' },
    { region: 'Europe', dataCenter: 'Chaos', world: 'Phantom' },
    { region: 'Europe', dataCenter: 'Chaos', world: 'Ragnarok' },
    { region: 'Europe', dataCenter: 'Chaos', world: 'Sagittarius' },
    { region: 'Europe', dataCenter: 'Chaos', world: 'Spriggan' },

    { region: 'Europe', dataCenter: 'Light', world: 'Alpha' },
    { region: 'Europe', dataCenter: 'Light', world: 'Lich' },
    { region: 'Europe', dataCenter: 'Light', world: 'Odin' },
    { region: 'Europe', dataCenter: 'Light', world: 'Phoenix' },
    { region: 'Europe', dataCenter: 'Light', world: 'Raiden' },
    { region: 'Europe', dataCenter: 'Light', world: 'Shiva' },
    { region: 'Europe', dataCenter: 'Light', world: 'Twintania' },
    { region: 'Europe', dataCenter: 'Light', world: 'Zodiark' },

    { region: 'Japan', dataCenter: 'Elemental', world: 'Aegis' },
    { region: 'Japan', dataCenter: 'Elemental', world: 'Atomos' },
    { region: 'Japan', dataCenter: 'Elemental', world: 'Carbuncle' },
    { region: 'Japan', dataCenter: 'Elemental', world: 'Garuda' },
    { region: 'Japan', dataCenter: 'Elemental', world: 'Gungnir' },
    { region: 'Japan', dataCenter: 'Elemental', world: 'Kujata' },
    { region: 'Japan', dataCenter: 'Elemental', world: 'Tonberry' },
    { region: 'Japan', dataCenter: 'Elemental', world: 'Typhon' },

    { region: 'Japan', dataCenter: 'Gaia', world: 'Alexander' },
    { region: 'Japan', dataCenter: 'Gaia', world: 'Bahamut' },
    { region: 'Japan', dataCenter: 'Gaia', world: 'Durandal' },
    { region: 'Japan', dataCenter: 'Gaia', world: 'Fenrir' },
    { region: 'Japan', dataCenter: 'Gaia', world: 'Ifrit' },
    { region: 'Japan', dataCenter: 'Gaia', world: 'Ridill' },
    { region: 'Japan', dataCenter: 'Gaia', world: 'Tiamat' },
    { region: 'Japan', dataCenter: 'Gaia', world: 'Ultima' },

    { region: 'Japan', dataCenter: 'Mana', world: 'Anima' },
    { region: 'Japan', dataCenter: 'Mana', world: 'Asura' },
    { region: 'Japan', dataCenter: 'Mana', world: 'Chocobo' },
    { region: 'Japan', dataCenter: 'Mana', world: 'Hades' },
    { region: 'Japan', dataCenter: 'Mana', world: 'Ixion' },
    { region: 'Japan', dataCenter: 'Mana', world: 'Masamune' },
    { region: 'Japan', dataCenter: 'Mana', world: 'Pandaemonium' },
    { region: 'Japan', dataCenter: 'Mana', world: 'Titan' },

    { region: 'Japan', dataCenter: 'Meteor', world: 'Belias' },
    { region: 'Japan', dataCenter: 'Meteor', world: 'Mandragora' },
    { region: 'Japan', dataCenter: 'Meteor', world: 'Ramuh' },
    { region: 'Japan', dataCenter: 'Meteor', world: 'Shinryu' },
    { region: 'Japan', dataCenter: 'Meteor', world: 'Unicorn' },
    { region: 'Japan', dataCenter: 'Meteor', world: 'Valefor' },
    { region: 'Japan', dataCenter: 'Meteor', world: 'Yojimbo' },
    { region: 'Japan', dataCenter: 'Meteor', world: 'Zeromus' },

    { region: 'Oceania', dataCenter: 'Materia', world: 'Bismarck' },
    { region: 'Oceania', dataCenter: 'Materia', world: 'Ravana' },
    { region: 'Oceania', dataCenter: 'Materia', world: 'Sephirot' },
    { region: 'Oceania', dataCenter: 'Materia', world: 'Sophia' },
    { region: 'Oceania', dataCenter: 'Materia', world: 'Zurvan' },
]
