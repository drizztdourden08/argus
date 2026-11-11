/* @layer renderer-app @kind component */
import { Inline, Select, Tag, Toggle } from '@drizztdourden08/tessera';
import { AUTO_SWITCH_LABEL, CAN_RECEIVE_LABEL, HANDOVER_LABEL, HANDOVER_OPTIONS } from '../accounts.constants';
import type { RuleChipsProps } from '../accounts.type';
import { handoverKind, handoverRule, ruleChips } from '../model';
import { PercentField } from './PercentField';

const RuleChips = (props: RuleChipsProps) => {
  const { account, rules } = props;
  const mine = rules.settings?.accounts.find((item) => item.key === (account.key ?? account.id));
  if (!mine) return <Inline gap="xs" wrap>{ruleChips(account.rules).map((chip) => <Tag key={chip}>{chip}</Tag>)}</Inline>;
  const change = (next: Partial<typeof mine.rules>) => void rules.changeAccount(mine.key, { rules: { ...mine.rules, ...next } });
  const kind = handoverKind(mine.rules.handover);
  return (
    <Inline gap="md" align="center" wrap>
      <Toggle size="sm" label={AUTO_SWITCH_LABEL} checked={mine.rules.autoSwitch} disabled={rules.busy} onChange={(autoSwitch) => change({ autoSwitch })} />
      {mine.rules.autoSwitch && <PercentField label={AUTO_SWITCH_LABEL} value={mine.rules.switchAt} disabled={rules.busy} onCommit={(switchAt) => change({ switchAt })} />}
      <Select size="sm" width="fit" aria-label={HANDOVER_LABEL} options={HANDOVER_OPTIONS} value={kind} onChange={(next) => change({ handover: handoverRule(next as typeof kind, mine.rules.handover) })} />
      {typeof mine.rules.handover === 'number' && <PercentField label={HANDOVER_LABEL} value={mine.rules.handover} disabled={rules.busy} onCommit={(handover) => change({ handover })} />}
      <Toggle size="sm" label={CAN_RECEIVE_LABEL} checked={mine.canReceive} disabled={rules.busy} onChange={(canReceive) => void rules.changeAccount(mine.key, { canReceive })} />
    </Inline>
  );
};

export { RuleChips };
