/* @layer renderer-app @kind component */
import { Button, Flex, Table, TableCell, TableRow, TableSection, Text } from '@drizztdourden08/tessera';
import type { TrashViewProps } from '../trash-props.type';
import { TrashLine } from './TrashLine';

const TrashView = (props: TrashViewProps) => {
  const { rows, now, selection, bulk, onBack, onPurge } = props;
  const order = rows.map((row) => row.key);
  const picked = rows.filter((row) => selection.has(row.key));
  const back = <Button size="sm" variant="secondary" onClick={onBack}>Back to the board</Button>;
  if (rows.length === 0) {
    return (
      <Flex direction="column" gap="md" align="start">
        {back}
        <Text tone="muted">Trash is empty.</Text>
      </Flex>
    );
  }
  return (
    <Flex direction="column" gap="md">
      <Flex gap="sm" align="center">
        {back}
        <Text variant="caption" tone="muted">{`${rows.length} in trash`}</Text>
      </Flex>
      {picked.length > 0 && (
        <Flex gap="sm" align="center" wrap role="toolbar" aria-label="Selected trash">
          <Text>{`${picked.length} selected`}</Text>
          <Button
            size="sm"
            variant="primary"
            onClick={() => {
              bulk.restore(picked);
              selection.clear();
            }}
          >
            Restore
          </Button>
          <Button size="sm" variant="ghost" onClick={selection.clear}>Clear selection</Button>
        </Flex>
      )}
      <Table aria-label="Trash">
        <TableSection head>
          <TableRow>
            <TableCell header>Select</TableCell>
            <TableCell header>Id</TableCell>
            <TableCell header>Kind</TableCell>
            <TableCell header>Text</TableCell>
            <TableCell header>Thread</TableCell>
            <TableCell header>Deleted</TableCell>
            <TableCell header>Restore</TableCell>
          </TableRow>
        </TableSection>
        <TableSection>
          {rows.map((row) => (
            <TrashLine key={row.key} row={row} now={now} selected={selection.has(row.key)} selection={selection} order={order} bulk={bulk} />
          ))}
        </TableSection>
      </Table>
      <Flex gap="sm" align="center">
        <Button size="sm" variant="danger" onClick={onPurge}>Empty trash...</Button>
        <Text variant="caption" tone="muted">Restorable until a purge removes them for good.</Text>
      </Flex>
    </Flex>
  );
};

export { TrashView };
