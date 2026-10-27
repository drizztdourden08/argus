/* @layer renderer-app @kind component */
import { Table, TableSection } from '@drizztdourden08/tessera';
import type { FeedTableProps } from '../overview.type';

const FeedTable = (props: FeedTableProps) => {
  const { label, children } = props;
  return (
    <Table aria-label={label}>
      <TableSection>{children}</TableSection>
    </Table>
  );
};

export { FeedTable };
