/* @layer renderer-app @kind component */
import { Table, TableCell, TableRow, TableSection } from '@drizztdourden08/tessera';
import type { NoticeDiffProps } from '../shared.type';
import { noticeRows } from '../model';

const NoticeDiff = (props: NoticeDiffProps) => {
  const rows = noticeRows(props.notice);
  if (rows.length === 0) return null;
  return (
    <Table aria-label="Your change and theirs">
      <TableSection head>
        <TableRow>
          <TableCell header>Field</TableCell>
          <TableCell header>Yours</TableCell>
          <TableCell header>Theirs</TableCell>
        </TableRow>
      </TableSection>
      <TableSection>
        {rows.map((row) => (
          <TableRow key={row.field}>
            <TableCell>{row.field}</TableCell>
            <TableCell>{row.mine}</TableCell>
            <TableCell>{row.theirs}</TableCell>
          </TableRow>
        ))}
      </TableSection>
    </Table>
  );
};

export { NoticeDiff };
