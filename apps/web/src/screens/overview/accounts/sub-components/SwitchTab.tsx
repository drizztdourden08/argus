/* @layer renderer-app @kind component */
import { Stack } from '@drizztdourden08/tessera';
import { isActive, ProcessPanel } from '../switch';
import type { SwitchTabProps } from '../next.type';
import { AccountCards } from './AccountCards';
import { NextStrip } from './NextStrip';

const SwitchTab = (props: SwitchTabProps) => {
  const { accounts, status, actions, doc, onOpen } = props;
  return (
    <Stack gap="md">
      <NextStrip accounts={accounts} status={status} actions={actions} />
      {isActive(doc) && <ProcessPanel doc={doc} accounts={accounts} />}
      <AccountCards accounts={accounts} onOpen={onOpen} />
    </Stack>
  );
};

export { SwitchTab };
