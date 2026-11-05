/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { HeaderPopover } from '../../../shell/HeaderPopover';
import { useRegisteredView } from './behavior/useRegisteredView';
import { NO_LOAD, REGISTERED_HINT } from './RegisteredStrip.constants';
import type { RegisteredStripProps } from './RegisteredStrip.type';
import { RegisteredChip } from './sub-components/RegisteredChip';
import { RegisteredLabel } from './sub-components/RegisteredLabel';

const RegisteredStrip = (props: RegisteredStripProps) => {
  const { boardKey } = props;
  const { rows, loads } = useRegisteredView(boardKey);
  return (
    <HeaderPopover trigger={<RegisteredLabel rows={rows} />} label="Registered threads" hint={REGISTERED_HINT}>
      {rows.length === 0 ? <Text variant="caption" tone="muted">No thread is registered on this board yet.</Text> : null}
      <Flex gap="xs" wrap>
        {rows.map((row) => (
          <RegisteredChip key={row.id} row={row} load={loads.get(row.thread) ?? NO_LOAD} />
        ))}
      </Flex>
    </HeaderPopover>
  );
};

export { RegisteredStrip };
