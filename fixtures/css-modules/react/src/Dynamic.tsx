import styles from "./Dynamic.module.css";

export function Dynamic({ size }: { size: string }) {
  return <p className={styles[size]} />;
}
