/* @layer renderer-app @kind component */
import { Button, DialogShell, Table, TableCell, TableRow, TableSection, Tag, Text } from '@drizztdourden08/tessera';
import { AttentionRows } from '../../tabs/kanban/sub-components/AttentionRows';
import { LEGEND_ENTRIES } from '../BoardMore.constants';

const LegendDialog = (props: { onClose: () => void }) => {
  const { onClose } = props;
  return (
    <DialogShell
      open
      size="lg"
      title="Legend"
      onClose={onClose}
      actions={<Button size="sm" variant="ghost" onClick={onClose}>Close</Button>}
    >
      <Table>
        <TableSection>
          {LEGEND_ENTRIES.map((entry) => (
            <TableRow key={entry.label}>
              <TableCell><Tag {...entry.look}>{entry.label}</Tag></TableCell>
              <TableCell><Text variant="subtitle">{entry.text}</Text></TableCell>
            </TableRow>
          ))}
          <AttentionRows />
        </TableSection>
      </Table>
    </DialogShell>
  );
};

export { LegendDialog };
