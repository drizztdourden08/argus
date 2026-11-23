/* @layer renderer-app @kind logic */
import { PORTFOLIO_SCOPE, useDoc } from '../../../../../data/react';
import type { AccountsSwitch } from '../../../../../data/types';
import { ACCOUNTS_STATE_COLL } from '../../accounts.constants';
import { SWITCH_DOC } from '../switch.constants';

const useSwitchFeed = (): AccountsSwitch | null => useDoc<AccountsSwitch>(PORTFOLIO_SCOPE, ACCOUNTS_STATE_COLL, SWITCH_DOC).doc;

export { useSwitchFeed };
