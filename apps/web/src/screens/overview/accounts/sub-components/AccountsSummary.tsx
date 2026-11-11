/* @layer renderer-app @kind component */
import { SignalStrip } from '@drizztdourden08/tessera';
import type { AccountsSummaryProps } from '../accounts.type';
import { signalsOf } from '../model';

const AccountsSummary = (props: AccountsSummaryProps) => (
  <SignalStrip items={signalsOf(props.accounts, props.drift)} wrap aria-label="Accounts summary" />
);

export { AccountsSummary };
