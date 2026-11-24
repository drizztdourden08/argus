/* @layer renderer-app @kind component */
import { DensityScope, Grid } from '@drizztdourden08/tessera';
import { CARD_MIN_WIDTH } from '../accounts.constants';
import { orderAccounts } from '../model';
import type { AccountCardsProps } from '../next.type';
import { AccountCard } from './AccountCard';

const AccountCards = (props: AccountCardsProps) => {
  const { accounts, onOpen } = props;
  return (
    <DensityScope density="compact">
      <Grid fit minColWidth={CARD_MIN_WIDTH} gap="sm" aria-label="Accounts">
        {orderAccounts(accounts).map((account) => <AccountCard key={account.id} account={account} onOpen={onOpen} />)}
      </Grid>
    </DensityScope>
  );
};

export { AccountCards };
