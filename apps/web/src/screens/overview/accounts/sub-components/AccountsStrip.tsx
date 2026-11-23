/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import { useNow } from '../../clock';
import { STRIP_COLUMNS } from '../accounts.constants';
import type { AccountsStripProps } from '../accounts.type';
import { AccountRow } from './AccountRow';
import { STRIP_TICK_MS } from './AccountsStrip.constants';
import './AccountsStrip.css';

const AccountsStrip = (props: AccountsStripProps) => {
  const { accounts, rules, confirmKey, switching, onSwitch, onCloseConfirm } = props;
  const now = useNow(STRIP_TICK_MS);
  return (
    <Box as="section" className="accounts-strip" aria-label="Accounts">
      <Box className="accounts-strip__row accounts-strip__head" aria-hidden>
        {STRIP_COLUMNS.map((column) => <Text key={column || 'actions'} variant="overline" tone="muted">{column}</Text>)}
      </Box>
      {accounts.map((account) => (
        <AccountRow
          key={account.id}
          account={account}
          rules={rules}
          now={now}
          confirming={confirmKey === account.id || confirmKey === account.key}
          switching={switching}
          onSwitch={onSwitch}
          onCloseConfirm={onCloseConfirm}
        />
      ))}
    </Box>
  );
};

export { AccountsStrip };
