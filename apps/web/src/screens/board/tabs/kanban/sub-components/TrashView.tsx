/* @layer renderer-app @kind component */
import { Button, Flex, Table, TableCell, TableRow, TableSection, Text } from '@drizztdourden08/tessera';
import { Panel } from '../../../../../shell';
import type { TrashViewProps } from '../trash-props.type';
import { TrashLine } from './TrashLine';

const TrashView = (props: TrashViewProps) => {
  const { rows, now, selection, bulk, onBack, onPurge } = props;
  const order = rows.map((row) => row.key);
  const picked = rows.filter((row) => selection.has(row.key));
  const back = <Button size="sm" onClick={onBack}>Back to the board</Button>;
  if (rows.length === 0) {
    return (
      <Panel title="Trash" note="0 in trash" tools={back}>
        <Text tone="muted">Trash is empty.</Text>
      </Panel>
    );
  }
  return (
    <Panel title="Trash" note={`${rows.length} in trash`} tools={back}>
      <Flex direction="column" gap="md">
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
    </Panel>
  );
};

export { TrashView };
