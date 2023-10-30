import React from 'react';
import style from './main.module.css';

export enum MarketForm {
    CONFIG,
    QUERY,
    EXCHANGE,
}

export function Header(
    { curForm, setForm }: { curForm: MarketForm, setForm: React.Dispatch<React.SetStateAction<MarketForm>> }
) {
    const headersInfo = [
        { form: MarketForm.CONFIG, title: 'Config' },
        { form: MarketForm.QUERY, title: 'Query' },
        { form: MarketForm.EXCHANGE, title: 'Exchange' },
    ];

    const setSelection = (newForm: MarketForm) => {
        setForm(newForm);
    }

    return (
        <div className={style.header}>
            {headersInfo.map(info => {
                return (
                    <HeaderSegment
                        key={info.title}
                        title={info.title}
                        isSelected={curForm == info.form}
                        setSelection={() => { setSelection(info.form); }}
                    />
                )
            })}
        </div>
    )
}

interface HeaderSegmentProps {
    title: string,
    isSelected: boolean,
    setSelection: () => void,
};

function HeaderSegment({ title, isSelected, setSelection }: HeaderSegmentProps) {
    return isSelected ? <div data-selected>{title}</div> : <div onClick={setSelection}>{title}</div>
}
