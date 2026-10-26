import styles from "./Button.module.css";

export function Button({ active }: { active: boolean }) {
  return (
    <button className={styles.button}>
      <span className={styles.primary} />
      <i className={styles["icon"]} />
      <b className={active ? styles.active : styles.missing} />
      <u className={styles.disabled + " " + styles.wrapper + " " + styles.scoped + " " + styles.broken} />
      <em style={{ margin: styles.gutter }} />
    </button>
  );
}
