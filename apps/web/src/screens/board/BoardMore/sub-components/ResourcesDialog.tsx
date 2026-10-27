/* @layer renderer-app @kind component */
import { DialogShell, Button, Flex, Table, TableCell, TableRow, TableSection, Tag, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import { RESOURCES_NOTE } from '../BoardMore.constants';
import type { MoreDialogsProps } from '../BoardMore.type';

const ResourcesDialog = (props: Pick<MoreDialogsProps, 'locks' | 'held' | 'onClose'>) => {
  const { locks, held, onClose } = props;
  return (
    <DialogShell
      open
      size="lg"
      title="Resources"
      onClose={onClose}
      actions={<Button size="sm" variant="ghost" onClick={onClose}>Close</Button>}
    >
      <Flex direction="column" gap="md">
        <Text variant="caption" tone="muted">{`${held} held. ${RESOURCES_NOTE}`}</Text>
        <Table>
          <TableSection head>
            <TableRow>
              {['Resource', 'Holder', 'Since', 'Queue', 'About'].map((name) => <TableCell key={name} header>{name}</TableCell>)}
            </TableRow>
          </TableSection>
          <TableSection>
            {locks.length === 0 && (
              <TableRow><TableCell colSpan={5} align="center"><Text tone="muted">Nothing claimed.</Text></TableCell></TableRow>
            )}
            {locks.map((lock) => (
              <TableRow key={lock.id}>
                <TableCell><Text mono>{lock.resource ?? lock.id}</Text></TableCell>
                <TableCell>{lock.holder ?? <Tag>free</Tag>}</TableCell>
                <TableCell>{lock.holder && lock.since ? <Age value={lock.since} /> : null}</TableCell>
                <TableCell>{(lock.queue ?? []).join(' › ')}</TableCell>
                <TableCell>{lock.description ?? ''}</TableCell>
              </TableRow>
            ))}
          </TableSection>
        </Table>
      </Flex>
    </DialogShell>
  );
};

export { ResourcesDialog };
