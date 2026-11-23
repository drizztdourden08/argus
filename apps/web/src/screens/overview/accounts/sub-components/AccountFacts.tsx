/* @layer renderer-app @kind component */
import { FactsPanel } from '@drizztdourden08/tessera';
import type { AccountFactsProps } from '../cards.type';
import { accountFactGroups } from '../model';

const AccountFacts = (props: AccountFactsProps) => {
  const { account, differs } = props;
  return <FactsPanel layout="terms" framed={false} label="Account facts" groups={accountFactGroups(account, differs)} />;
};

export { AccountFacts };
