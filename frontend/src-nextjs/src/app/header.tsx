import React, { useState } from 'react';
import style from './header.module.css';
import { MarketForm } from './page';

export default function Header(
    { setForm }: { setForm: React.Dispatch<React.SetStateAction<MarketForm>> }
) {
    let [curSelected, setCurSelected] = useState(MarketForm.QUERY);

    const headersInfo = [
        { form: MarketForm.QUERY, title: 'Query' },
        { form: MarketForm.FIRMAMENT, title: 'Firmament' },
    ];

    const setSelection = (newForm: MarketForm) => {
        setCurSelected(newForm);
        setForm(newForm);
    }

    return (
        <div className={style.header}>
            {headersInfo.map(info => {
                return (
                    <HeaderSegment
                        key={info.title}
                        title={info.title}
                        isSelected={curSelected == info.form}
                        setSelection={() => setSelection(info.form)}
                    />
                )
            })}
        </div>
    )
}

type HeaderSegmentProps = {
    title: string,
    isSelected: boolean,
    setSelection: () => void,
};

function HeaderSegment({ title, isSelected, setSelection }: HeaderSegmentProps) {
    return isSelected ? <div data-selected>{title}</div> : <div onClick={setSelection}>{title}</div>
}
