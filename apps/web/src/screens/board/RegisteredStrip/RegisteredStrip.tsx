/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useRegisteredView } from './behavior/useRegisteredView';
import { NO_LOAD } from './RegisteredStrip.constants';
import type { RegisteredStripProps } from './RegisteredStrip.type';
import { RegisteredChip } from './sub-components/RegisteredChip';

const RegisteredStrip = (props: RegisteredStripProps) => {
  const { boardKey } = props;
  const { rows, loads } = useRegisteredView(boardKey);
  if (rows.length === 0) return null;
  const waking = rows.filter((row) => row.state === 'waking').length;
  const faults = rows.filter((row) => row.state === 'fault').length;
  return (
    <Flex gap="sm" align="center" wrap role="status" aria-label="Registered threads">
      <Text variant="label">Registered {rows.length}</Text>
      {waking > 0 ? <Text variant="label" tone="warning">{waking} waking</Text> : null}
      {faults > 0 ? <Text variant="label" tone="danger">{faults} {faults > 1 ? 'faults' : 'fault'}</Text> : null}
      {rows.map((row) => (
        <RegisteredChip key={row.id} row={row} load={loads.get(row.thread) ?? NO_LOAD} />
      ))}
    </Flex>
  );
};

export { RegisteredStrip };
