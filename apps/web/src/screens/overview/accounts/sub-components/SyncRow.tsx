/* @layer renderer-app @kind component */
import { Button, Inline, Text } from '@drizztdourden08/tessera';
import { DRIFT_TIP } from '../accounts.constants';
import type { SyncRowProps } from '../accounts.type';

const SyncRow = (props: SyncRowProps) => {
  const { drift, actions } = props;
  return (
    <Inline gap="md" align="center" wrap>
      <Text size="md" weight={600}>Sidebar sync</Text>
      <Text variant="subtitle" size="md" title={DRIFT_TIP}>{drift === 0 ? 'Sidebars match' : `${drift} to sync`}</Text>
      <Button size="sm" disabled={actions.busy} onClick={actions.sync}>Sync now</Button>
    </Inline>
  );
};

export { SyncRow };
