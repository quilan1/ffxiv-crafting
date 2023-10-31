import Image from 'next/image';
import { allDataCenters } from '../(universalis)/data-center';
import styles from './config.module.css';
import startingExample from './startingExample.png';
import craftExample from './craftExample.png';
import purchaseExample from './purchaseExample.png';
import { useAppContext } from '../context';

export function ConfigContainer() {
    return (
        <div className={styles.config}>
            <ConfigStatus />
            <ConfigText />
        </div>
    )
}

function ConfigStatus() {
    const { configState: { homeworld } } = useAppContext();

    const regions = [...allDataCenters
        .reduce((set, info) => set.add(info.region), new Set<string>())
    ];
    const dataCenters = (region: string) => {
        return [...allDataCenters
            .filter(info => info.region === region)
            .reduce((set, info) => set.add(info.dataCenter), new Set<string>())
        ];
    }
    const worlds = (dataCenter: string) => {
        return [...allDataCenters
            .filter(info => info.dataCenter === dataCenter)
            .reduce((set, info) => set.add(info.world), new Set<string>())
        ]
    }

    const allKeys: string[] = [];
    regions.forEach(region => {
        allKeys.push(`[ Region: ${region} ]`);
        dataCenters(region).forEach(dataCenter => {
            allKeys.push(`= ${dataCenter} =`);
            worlds(dataCenter).forEach(world => {
                allKeys.push(world);
            })
        })
    });

    const onChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        if (e.target.value.startsWith('=') || e.target.value.startsWith('[')) return;
        homeworld.value = e.target.value;
    };

    return (
        <div className={styles.configHeader}>
            <div>
                <label>Homeworld:</label>
                <select value={homeworld.value} onChange={onChange}>
                    {allKeys.map(key => {
                        return (key.startsWith('=') || key.startsWith('['))
                            ? <option key={key} value={key} style={{ fontWeight: 'bold' }}>{key}</option>
                            : <option key={key} value={key}>{key}</option>
                    })}
                </select>
            </div>
        </div >
    );
}

function ConfigText() {
    return (
        <div className={styles.usage}>
            <div>
                <h2>Overview</h2>
                This website is intended as a way of querying market deals in FFXIV that can be made by crafting
                various materials. Query strings determine which items will be fetched from Universalis for current
                marketplace information. With this, the calculated optimal crafting, expected profit and
                individual purchases are determined for each item.
            </div>
            <div>
                <h3>Quick Start</h3>
                <div>
                    <ol>
                        <li>
                            Select your FFXIV homeworld from the dropdown of the <span style={{ fontWeight: 'bold' }}>Config Tab</span>.
                            Top of this page.
                        </li>
                        <li>Navigate to the <span style={{ fontWeight: 'bold' }}>Query Tab</span>.</li>
                        <li>
                            Select one of the query patterns from
                            the <span style={{ fontWeight: 'bold' }}>&apos;Examples&apos;</span> dropdown then
                            press <span style={{ fontWeight: 'bold' }}>&apos;Fetch&apos;</span>.
                            <div><Image src={startingExample} alt="Choosing an example from the Query tab" priority={true} /></div>
                        </li>
                        <li>
                            Choose a craft with a high expected profit, and click the checkmark next to the top level item of that
                            craft.
                            <div><Image src={craftExample} alt="Identifying a craft with a high profit" priority={true} /></div>
                        </li>
                        <li>
                            Ingredient purchase listings will be shown below with retainer names, ordered by data center.
                            <div><Image
                                src={purchaseExample}
                                alt="Purchase information for items"
                                style={{ border: '2px solid #404040' }}
                                priority={true}
                            /></div>
                        </li>
                    </ol>
                </div>
            </div>
            <h3>References</h3>
            <div>
                <ul>
                    <li><a href="https://github.com/quilan1/ffxiv-crafting#query-language">Query string format reference</a></li>
                    <li><a href="https://universalis.app/">Universalis App</a></li>
                </ul>
            </div>
        </div >
    )
}
