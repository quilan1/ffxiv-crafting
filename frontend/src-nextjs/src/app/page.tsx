'use client';
import React, { useState } from 'react';
import Header from './header'
import styles from './index.module.css';
import { QueryContainer } from './(query)/query-container';
import { FirmamentContainer } from './(firmament)/firmament-container';

export enum MarketForm {
    QUERY,
    FIRMAMENT,
}

export default function Home() {
    let [curForm, setForm] = useState(MarketForm.QUERY);

    let form = <div className={styles.error}>Invalid MarketForm value: {curForm}</div>;
    if (curForm == MarketForm.QUERY) {
        form = <QueryContainer />;
    } else if (curForm == MarketForm.FIRMAMENT) {
        form = <FirmamentContainer />;
    }

    return (
        <main className={styles.main}>
            <Header setForm={setForm} />
            <div className={styles.contentContainer}>{form}</div>
        </main>
    )
}
