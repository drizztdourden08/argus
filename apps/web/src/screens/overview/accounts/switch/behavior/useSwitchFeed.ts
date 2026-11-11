/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { PORTFOLIO_SCOPE, useCollection, useDoc } from '../../../../../data/react';
import type { AccountsCycle, AccountsSwitch, HandoverDoc } from '../../../../../data/types';
import { ACCOUNTS_STATE_COLL } from '../../accounts.constants';
import { CYCLE_DOC, HANDOVERS_COLL, SWITCH_DOC } from '../switch.constants';
import type { SwitchFeed } from '../switch.type';

const useSwitchFeed = (): SwitchFeed => {
  const doc = useDoc<AccountsSwitch>(PORTFOLIO_SCOPE, ACCOUNTS_STATE_COLL, SWITCH_DOC).doc;
  const cycle = useDoc<AccountsCycle>(PORTFOLIO_SCOPE, ACCOUNTS_STATE_COLL, CYCLE_DOC).doc;
  const all = useCollection<HandoverDoc>(PORTFOLIO_SCOPE, HANDOVERS_COLL).docs;
  const handovers = useMemo(() => (cycle ? all.filter((item) => item.cycle === cycle.id) : []), [all, cycle]);
  return { doc, cycle, handovers };
};

export { useSwitchFeed };
