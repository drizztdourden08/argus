/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { accountKey, dragAccount } from '../model';
import type { AccountCardProps } from '../next.type';
import { AccountBrief } from './AccountBrief';
import './AccountCard.css';

const AccountCard = (props: AccountCardProps) => {
  const { account, onOpen } = props;
  const key = accountKey(account);
  const active = account.active === true;
  return (
    <Card className="account-card" draggable onDragStart={dragAccount(key)} onActivate={() => onOpen(key)} edge={active ? 'left' : undefined} edgeTone={active ? 'info' : undefined}>
      <AccountBrief account={account} />
    </Card>
  );
};

export { AccountCard };
