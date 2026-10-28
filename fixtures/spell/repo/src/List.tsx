// Lists what we recieve.
export const List = ({ rows }: { rows: string[] }) => {
  const recievedRows = rows.filter(Boolean);
  return (
    <ul aria-label="the lenght list">
      {recievedRows.map((r) => (
        <li key={r}>{r}</li>
      ))}
    </ul>
  );
};
