import styles from './divider.module.css';

export const HDivider = ({ flipWhenSmall }: { flipWhenSmall?: boolean }) => {
    return <Divider horizontal={true} flipWhenSmall={flipWhenSmall} />;
}

export const VDivider = ({ flipWhenSmall }: { flipWhenSmall?: boolean }) => {
    return <Divider horizontal={false} flipWhenSmall={flipWhenSmall} />;
}

export const Divider = ({ horizontal, flipWhenSmall }: { horizontal: boolean, flipWhenSmall?: boolean }) => {
    const style = [horizontal ? styles.hdivider : styles.vdivider, flipWhenSmall ? styles.flipped : '']
        .filter(s => s.length > 0)
        .join(' ');

    return <div className={style}><span></span></div>
}

