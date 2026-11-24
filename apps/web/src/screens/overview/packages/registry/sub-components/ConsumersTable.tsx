/* @layer renderer-app @kind component */
import { Flex, Table, TableCell, TableRow, TableSection, Tag, Text } from '@drizztdourden08/tessera';
import type { ConsumersTableProps } from '../registry.type';

const ConsumersTable = (props: ConsumersTableProps) => {
  const { consumers } = props;
  return (
    <Table aria-label="Consumers switched on">
      <TableSection head>
        <TableRow>
          <TableCell header>Consumer</TableCell>
          <TableCell header>Packages</TableCell>
        </TableRow>
      </TableSection>
      <TableSection>
        {consumers.map((consumer) => (
          <TableRow key={consumer.path}>
            <TableCell><Text mono>{consumer.path}</Text></TableCell>
            <TableCell>
              <Flex gap="xs" wrap>
                {consumer.packages.map((name) => <Tag key={name} pill>{name}</Tag>)}
              </Flex>
            </TableCell>
          </TableRow>
        ))}
      </TableSection>
    </Table>
  );
};

export { ConsumersTable };
