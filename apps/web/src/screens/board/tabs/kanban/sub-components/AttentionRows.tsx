/* @layer renderer-app @kind component */
import { SignalStrip, TableCell, TableRow, Text } from '@drizztdourden08/tessera';
import { ATTENTION } from '../attention';

const AttentionRows = () => (
  <>
    {ATTENTION.map((spec) => (
      <TableRow key={spec.kind}>
        <TableCell><SignalStrip items={[{ id: spec.kind, icon: spec.icon, label: spec.label, tone: spec.color }]} iconOnly aria-label={spec.label} /></TableCell>
        <TableCell><Text variant="subtitle">{spec.text}</Text></TableCell>
      </TableRow>
    ))}
  </>
);

export { AttentionRows };
