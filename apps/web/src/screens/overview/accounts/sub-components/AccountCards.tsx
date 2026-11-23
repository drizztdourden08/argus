/* @layer renderer-app @kind component */
import { Grid } from '@drizztdourden08/tessera';
import { useNow } from '../../clock';
import { CARD_MIN_WIDTH, CARDS_TICK_MS } from '../accounts.constants';
import type { AccountCardsProps } from '../cards.type';
import { orderAccounts } from '../model';
import { AccountCard } from './AccountCard';

const AccountCards = (props: AccountCardsProps) => {
  const { accounts, rules, compare, confirmKey, switching, onSwitch } = props;
  const now = useNow(CARDS_TICK_MS);
  const current = accounts.find((account) => account.active === true) ?? null;
  return (
    <Grid fit minColWidth={CARD_MIN_WIDTH} gap="md" aria-label="Accounts">
      {orderAccounts(accounts).map((account) => (
        <AccountCard
          key={account.id}
          account={account}
          current={current}
          rules={rules}
          compare={compare}
          now={now}
          confirming={confirmKey === account.id || confirmKey === account.key}
          switching={switching}
          onSwitch={onSwitch}
        />
      ))}
    </Grid>
  );
};

export { AccountCards };
