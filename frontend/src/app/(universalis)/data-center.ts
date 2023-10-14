export const dataCenterOf = (world: string): string => {
    return {
        'Halicarnassus': 'Dynamis',
        'Maduin': 'Dynamis',
        'Marilith': 'Dynamis',
        'Seraph': 'Dynamis',

        'Adamantoise': 'Aether',
        'Cactuar': 'Aether',
        'Faerie': 'Aether',
        'Gilgamesh': 'Aether',
        'Jenova': 'Aether',
        'Midgardsormr': 'Aether',
        'Sargatanas': 'Aether',
        'Siren': 'Aether',

        'Balmung': 'Crystal',
        'Brynhildr': 'Crystal',
        'Coeurl': 'Crystal',
        'Diabolos': 'Crystal',
        'Goblin': 'Crystal',
        'Malboro': 'Crystal',
        'Mateus': 'Crystal',
        'Zalera': 'Crystal',

        'Behemoth': 'Primal',
        'Excalibur': 'Primal',
        'Exodus': 'Primal',
        'Famfrit': 'Primal',
        'Hyperion': 'Primal',
        'Lamia': 'Primal',
        'Leviathan': 'Primal',
        'Ultros': 'Primal',
    }[world] ?? "<UNKNOWN>" as string;
}