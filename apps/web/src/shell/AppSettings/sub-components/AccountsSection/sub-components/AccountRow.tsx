/* @layer renderer-app @kind component */
import { Card, Inline, Select, Stack, Tag, Text, TextInput, Toggle } from '@drizztdourden08/tessera';
import { MAX_PERCENT, MIN_PERCENT } from '../../../../../screens/overview/accounts/accounts.constants';
import { handoverKind, handoverRule } from '../../../../../screens/overview/accounts/model';
import { DEFAULT_COLOR, HANDOVER_CHOICES, NEW_KEY_HINT } from '../AccountsSection.constants';
import type { AccountRowProps } from '../AccountsSection.type';
import { NumberField } from './NumberField';

const loginText = (saved: boolean | undefined, at: string | null | undefined): string => {
  if (!saved) return 'Desktop login not saved';
  return at ? `Desktop login saved ${new Date(at).toLocaleString('en-CA')}` : 'Desktop login saved';
};

const AccountRow = (props: AccountRowProps) => {
  const { item, doc, isNew, onChange } = props;
  const rules = item.rules;
  const kind = handoverKind(rules.handover);
  const setRules = (next: Partial<typeof rules>) => onChange({ rules: { ...rules, ...next } });
  const color = item.color.startsWith('#') ? (item.color as `#${string}`) : DEFAULT_COLOR;
  return (
    <Card>
      <Stack gap="sm">
        <Inline gap="sm" align="center" wrap>
          <Tag variant="custom" color={color} pill>{item.label.slice(0, 1).toUpperCase() || '?'}</Tag>
          {isNew
            ? <TextInput size="sm" aria-label="Key" placeholder={NEW_KEY_HINT} value={item.key} onChange={(event) => onChange({ key: event.target.value.trim() })} />
            : <Text weight={600}>{item.key}</Text>}
          <TextInput size="sm" aria-label="Label" placeholder="Label" value={item.label} onChange={(event) => onChange({ label: event.target.value })} />
          <TextInput size="sm" aria-label="Color" mono value={item.color} onChange={(event) => onChange({ color: event.target.value })} />
        </Inline>
        <TextInput size="sm" aria-label="CLI folder" mono placeholder="CLI folder" value={item.cliDir} onChange={(event) => onChange({ cliDir: event.target.value })} />
        <Text variant="caption" tone="muted">{loginText(doc?.desktopLogin?.saved, doc?.desktopLogin?.savedAt)}</Text>
        <Inline gap="md" align="end" wrap>
          <Toggle size="sm" label="Can be switched to" checked={item.canReceive} onChange={(canReceive) => onChange({ canReceive })} />
          <Toggle size="sm" label="Auto-switch" checked={rules.autoSwitch} onChange={(autoSwitch) => setRules({ autoSwitch })} />
          {rules.autoSwitch && <NumberField label="Switch at (%)" min={MIN_PERCENT} max={MAX_PERCENT} value={rules.switchAt} onChange={(switchAt) => setRules({ switchAt })} />}
          <Select size="sm" width="fit" aria-label="Handover" options={HANDOVER_CHOICES} value={kind} onChange={(next) => setRules({ handover: handoverRule(next as typeof kind, rules.handover) })} />
          {typeof rules.handover === 'number' && <NumberField label="Handover at (%)" min={MIN_PERCENT} max={MAX_PERCENT} value={rules.handover} onChange={(handover) => setRules({ handover })} />}
        </Inline>
      </Stack>
    </Card>
  );
};

export { AccountRow };
