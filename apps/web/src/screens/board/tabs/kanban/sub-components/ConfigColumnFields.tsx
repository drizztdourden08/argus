/* @layer renderer-app @kind component */
import { Checkbox, Field, Flex, Select, Text, TextInput } from '@drizztdourden08/tessera';
import { ACCENTS, LANES, configToggleList } from '../model';
import type { ConfigColumnFieldsProps } from '../config-editor.type';
import { ConfigListInput } from './ConfigListInput';

const NO_ACCENT = 'none';

const ACCENT_OPTIONS = [{ value: NO_ACCENT, label: 'No accent' }, ...ACCENTS.map((accent) => ({ value: accent, label: accent }))];

const ConfigColumnFields = (props: ConfigColumnFieldsProps) => {
  const { column, rules, onChange } = props;
  const attached = column.rules ?? [];
  return (
    <Flex direction="column" gap="md">
      <Flex gap="md">
        <Field label="Column id" width="sm">
          <TextInput value={column.id} onChange={(event) => onChange({ ...column, id: event.target.value })} />
        </Field>
        <Field label="Name">
          <TextInput value={column.name} onChange={(event) => onChange({ ...column, name: event.target.value })} />
        </Field>
        <Field label="Accent" width="sm">
          <Select options={ACCENT_OPTIONS} value={column.accent ?? NO_ACCENT} onChange={(accent) => onChange({ ...column, accent: accent === NO_ACCENT ? null : accent })} />
        </Field>
      </Flex>
      <Text variant="label">Lanes</Text>
      <Flex gap="md" wrap>
        {LANES.map((lane) => (
          <Checkbox key={lane} label={lane} checked={column.match.lanes.includes(lane)} onChange={(on) => onChange({ ...column, match: { ...column.match, lanes: configToggleList(column.match.lanes, lane, on) } })} />
        ))}
      </Flex>
      <ConfigListInput label="Stages" hint="Comma separated. Empty means every stage." value={column.match.stages ?? []} onCommit={(stages) => onChange({ ...column, match: { ...column.match, stages } })} />
      <Text variant="label">Rules</Text>
      <Flex gap="md" wrap>
        {rules.map((rule) => (
          <Checkbox key={rule.id} label={rule.id} checked={attached.includes(rule.id)} onChange={(on) => onChange({ ...column, rules: configToggleList(attached, rule.id, on) })} />
        ))}
        {rules.length === 0 && <Text variant="caption" tone="muted">No rules yet.</Text>}
      </Flex>
    </Flex>
  );
};

export { ConfigColumnFields };
