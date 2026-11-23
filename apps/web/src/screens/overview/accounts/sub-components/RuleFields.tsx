/* @layer renderer-app @kind component */
import { Field, Inline, Select, Toggle } from '@drizztdourden08/tessera';
import {
  ASK_AT_LABEL, AUTO_SWITCH_HINT, AUTO_SWITCH_LABEL, CAN_RECEIVE_HINT, CAN_RECEIVE_LABEL, HANDOVER_HINT, HANDOVER_LABEL, HANDOVER_OPTIONS, SWITCH_AT_LABEL,
} from '../accounts.constants';
import type { RuleFieldsProps } from '../cards.type';
import { handoverKind, handoverRule } from '../model';
import { PercentField } from './PercentField';

const RuleFields = (props: RuleFieldsProps) => {
  const { accountKey, rules } = props;
  const mine = rules.settings?.accounts.find((item) => item.key === accountKey);
  if (!mine) return null;
  const change = (next: Partial<typeof mine.rules>) => void rules.changeAccount(mine.key, { rules: next });
  const kind = handoverKind(mine.rules.handover);
  return (
    <Inline gap="lg" align="start" wrap>
      <Field size="sm" label={AUTO_SWITCH_LABEL} hint={AUTO_SWITCH_HINT}>
        <Toggle size="sm" checked={mine.rules.autoSwitch} disabled={rules.busy} onChange={(autoSwitch) => change({ autoSwitch })} />
      </Field>
      {mine.rules.autoSwitch && (
        <Field size="sm" label={SWITCH_AT_LABEL}>
          <PercentField label={SWITCH_AT_LABEL} value={mine.rules.switchAt} disabled={rules.busy} onCommit={(switchAt) => change({ switchAt })} />
        </Field>
      )}
      <Field size="sm" label={HANDOVER_LABEL} hint={HANDOVER_HINT}>
        <Select size="sm" width="fit" options={HANDOVER_OPTIONS} value={kind} onChange={(next) => change({ handover: handoverRule(next as typeof kind, mine.rules.handover) })} />
      </Field>
      {typeof mine.rules.handover === 'number' && (
        <Field size="sm" label={ASK_AT_LABEL}>
          <PercentField label={ASK_AT_LABEL} value={mine.rules.handover} disabled={rules.busy} onCommit={(handover) => change({ handover })} />
        </Field>
      )}
      <Field size="sm" label={CAN_RECEIVE_LABEL} hint={CAN_RECEIVE_HINT}>
        <Toggle size="sm" checked={mine.canReceive} disabled={rules.busy} onChange={(canReceive) => void rules.changeAccount(mine.key, { canReceive })} />
      </Field>
    </Inline>
  );
};

export { RuleFields };
