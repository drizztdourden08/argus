/* @layer renderer-app @kind logic */
import { readRules, removeRule, runRule, updateRule, useAction } from '../../../data/overlord';
import type { OverlordRule } from '../../../data/overlord';
import { useRemoteRead } from '../../../data/remote';
import { RULES_POLL_MS } from '../OverlordChat.constants';

const useRules = () => {
  const read = useRemoteRead(readRules, RULES_POLL_MS);
  const action = useAction(read.reload);
  return {
    rules: read.data?.rules ?? [],
    problem: read.error,
    reload: read.reload,
    action,
    toggle: (rule: OverlordRule, enabled: boolean) => action.run(() => updateRule(rule.id, { enabled })),
    run: (rule: OverlordRule) => action.run(() => runRule(rule.id)),
    remove: (rule: OverlordRule) => action.run(() => removeRule(rule.id)),
  };
};

export { useRules };
