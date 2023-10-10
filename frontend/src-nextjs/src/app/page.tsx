'use client';
import React, { useState } from 'react';
import Header from './header'
import styles from './main.module.css';
import { QueryContainer } from './(query)/query';
import { FirmamentContainer } from './(firmament)/firmament-container';
import QueryContextProvider from './(query)/context';

export enum MarketForm {
    QUERY,
    FIRMAMENT,
}

export default function Home() {
    const [curForm, setForm] = useState(MarketForm.QUERY);

    let form = <div className={styles.error}>Invalid MarketForm value: {curForm}</div>;
    if (curForm == MarketForm.QUERY) {
        form = <QueryContainer />;
    } else {
        form = <FirmamentContainer />;
    }

    return (
        <main className={styles.main}>
            <QueryContextProvider>
                <Header setForm={setForm} />
                <div className={styles.contentContainer}>{form}</div>
            </QueryContextProvider>
        </main>
    )
}
