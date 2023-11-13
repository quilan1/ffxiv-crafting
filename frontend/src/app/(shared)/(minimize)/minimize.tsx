import { Signal } from "@/app/(util)/signal";
import triangle from '../triangle.png';
import Image from 'next/image';
import styles from './minimize.module.css';

export function Minimize({ isMinimized }: { isMinimized: Signal<boolean> }) {
    const style = [styles.triangle, isMinimized.value ? styles.minimized : '']
        .filter(s => s.length > 0)
        .join(' ');

    const onClick = () => isMinimized.value = !isMinimized.value;
    return (
        <div className={styles.minOptions} onClick={onClick}>
            <Image className={style} src={triangle} alt="minimize" />
        </div>
    )
}