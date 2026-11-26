/* @layer renderer-app @kind component */
import { DensityScope, Inline } from '@drizztdourden08/tessera';
import { orderAccounts } from '../model';
import { CARDS_LABEL } from '../next.constants';
import type { AccountCardsProps } from '../next.type';
import { AccountCard } from './AccountCard';

const AccountCards = (props: AccountCardsProps) => {
  const { accounts, onOpen } = props;
  return (
    <DensityScope density="compact">
      <Inline gap="sm" align="stretch" justify="center" wrap role="group" aria-label={CARDS_LABEL}>
        {orderAccounts(accounts).map((account) => <AccountCard key={account.id} account={account} onOpen={onOpen} />)}
      </Inline>
    </DensityScope>
  );
};

export { AccountCards };
