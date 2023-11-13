import styles from './main.module.css';
import './global.css';

export const metadata = {
    title: 'FFXIV Crafting',
    description: 'An app that fetches marketplace information for FFXIV',
    robots: 'noindex,nofollow',
}

export default function RootLayout({
    children,
}: {
    children: React.ReactNode
}) {
    return (
        <html lang="en">
            <body className={styles.body}>{children}</body>
        </html>
    )
}
