/* @layer renderer-app @kind component */
import { TableCell, TableRow, Tag, Text } from '@drizztdourden08/tessera';
import { ATTENTION } from '../attention';

const AttentionRows = () => (
  <>
    {ATTENTION.map((spec) => (
      <TableRow key={spec.kind}>
        <TableCell><Tag variant="urgency" color={spec.color} icon={spec.icon}>{spec.label}</Tag></TableCell>
        <TableCell><Text variant="subtitle">{spec.text}</Text></TableCell>
      </TableRow>
    ))}
  </>
);

export { AttentionRows };
