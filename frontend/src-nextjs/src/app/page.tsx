'use client';
import React, { useState } from 'react';
import Header from './header'
import styles from './main.module.css';
import { QueryContainer } from './(query)/query';
import { FirmamentContainer } from './(firmament)/firmament';
import { QueryContextProvider } from './(query)/context';
import { FirmamentContextProvider } from './(firmament)/context';

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
                <FirmamentContextProvider>
                    <Header setForm={setForm} />
                    <div className={styles.contentContainer}>{form}</div>
                </FirmamentContextProvider>
            </QueryContextProvider>
        </main>
    )
}
