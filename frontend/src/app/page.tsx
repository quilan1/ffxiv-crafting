'use client';
import React, { useState } from 'react';
import { Header, MarketForm } from './header'
import styles from './main.module.css';
import { QueryContainer } from './(query)/query';
import { ExchangeContainer } from './(exchange)/exchange';
import { ConfigContainer } from './(config)/config';

export default function Home() {
    const [curForm, setForm] = useState(MarketForm.CONFIG);

    let form = <div className={styles.error}>Invalid MarketForm value: {curForm}</div>;

    switch (curForm) {
        case MarketForm.CONFIG:
            form = <ConfigContainer />;
            break;
        case MarketForm.QUERY:
            form = <QueryContainer />;
            break;
        case MarketForm.EXCHANGE:
            form = <ExchangeContainer />;
            break;
        default:
            const _check: never = curForm;
            console.error(`Missing form: ${_check}`);
    }

    return (
        <main className={styles.main}>
            <Header curForm={curForm} setForm={setForm} />
            <div className={styles.contentContainer}>{form}</div>
        </main>
    )
}
