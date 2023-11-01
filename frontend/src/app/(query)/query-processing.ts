interface PreparedQuery {
    label: string,
    query: string,
    count?: string,
    limit?: string,
    minVelocity?: string,
}

export const preparedQueries: PreparedQuery[] = [
    { label: 'Level 90 Crafting Mats', query: ':rlevel 90, :cat !Metal|Lumber|Leather|Stone|Cloth|Reagent', count: '20', limit: '10' },
    { label: 'Quick Mats', query: ':rlevel 1|90, :cat !Metal|Lumber|Leather|Stone|Cloth|Reagent', count: '20', limit: '16', minVelocity: '50.0' },
    { label: 'Popular Housing', query: ':cat !Ceiling Light|Door|Flooring|Furnishing|Interior Wall|Placard|Rug|Table|Tabletop|Window|Exterior Wall|Exterior Wall Decoration|Fence|Outdoor Furnishing|Roof|Roof Decoration|Wall-mounted', count: '5', limit: '16', minVelocity: '5.0' },
    { label: 'Cosmetics', query: ':rlevel 1|90, :ilevel 1, :cat !Head|Body|Hands|Legs|Feet', count: '2', limit: '16', minVelocity: '1.0' },
    { label: 'Skybuilders\' Crafts', query: ':rlevel 80, :name Grade 4 Skybuilders\'', count: '50', limit: '4' },
    { label: 'White Scrips', query: ':name ^Rarefied, :rlevel 50|89', count: '40', limit: '4' },
    { label: 'Purple Scrips', query: ':name ^Rarefied, :rlevel 90', count: '40', limit: '4' },
    { label: 'Maps', query: ':name Timeworn .*skin Map', count: '1' },
];

export const defaultQuery = preparedQueries[0];
