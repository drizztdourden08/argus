/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useRegisteredView } from './behavior/useRegisteredView';
import { NO_LOAD } from './RegisteredStrip.constants';
import type { RegisteredStripProps } from './RegisteredStrip.type';
import { RegisteredChip } from './sub-components/RegisteredChip';

const RegisteredStrip = (props: RegisteredStripProps) => {
  const { boardKey } = props;
  const { rows, loads } = useRegisteredView(boardKey);
  if (rows.length === 0) return <Text variant="caption" tone="muted">No thread is registered on this board yet.</Text>;
  return (
    <Flex gap="xs" wrap aria-label="Registered threads">
      {rows.map((row) => (
        <RegisteredChip key={row.id} row={row} load={loads.get(row.thread) ?? NO_LOAD} />
      ))}
    </Flex>
  );
};

export { RegisteredStrip };
