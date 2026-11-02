/* @layer renderer-app @kind component */
import { TableCell, TableRow, Text } from '@drizztdourden08/tessera';
import { depColor } from '../../../../data/deps';
import { Age, DepTag } from '../../../../shell';
import type { DepRowProps } from '../deps.type';
import { changedAt } from '../model';
import { DepEndCell } from './DepEndCell';
import { DepWaitCell } from './DepWaitCell';

const DepRow = (props: DepRowProps) => {
  const { doc, now } = props;
  return (
    <TableRow>
      <TableCell><DepEndCell end={doc.from} /></TableCell>
      <TableCell><DepEndCell end={doc.to} /></TableCell>
      <TableCell><DepTag color={depColor(doc.state)} title={doc.reason ?? doc.title}>{doc.state}</DepTag></TableCell>
      <TableCell><DepWaitCell doc={doc} now={now} /></TableCell>
      <TableCell><Text size="md" tone="dim" numeric><Age value={changedAt(doc)} /></Text></TableCell>
    </TableRow>
  );
};

export { DepRow };
