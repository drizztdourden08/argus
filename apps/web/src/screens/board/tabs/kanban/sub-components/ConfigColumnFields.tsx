/* @layer renderer-app @kind component */
import { Checkbox, Field, Flex, Select, Text, TextInput } from '@drizztdourden08/tessera';
import { LANES, configToggleList } from '../model';
import type { ConfigColumnFieldsProps } from '../config-editor.type';
import { ACCENT_OPTIONS, NO_ACCENT } from './ConfigColumnFields.constants';
import { ConfigListInput } from './ConfigListInput';

const ConfigColumnFields = (props: ConfigColumnFieldsProps) => {
  const { column, onChange } = props;
  return (
    <Flex direction="column" gap="md">
      <Field label="Column id">
        <TextInput value={column.id} onChange={(event) => onChange({ ...column, id: event.target.value })} />
      </Field>
      <Field label="Name">
        <TextInput value={column.name} onChange={(event) => onChange({ ...column, name: event.target.value })} />
      </Field>
      <Field label="Accent">
        <Select options={ACCENT_OPTIONS} value={column.accent ?? NO_ACCENT} onChange={(accent) => onChange({ ...column, accent: accent === NO_ACCENT ? null : accent })} />
      </Field>
      <Text variant="label">Lanes</Text>
      <Flex gap="md" wrap>
        {LANES.map((lane) => (
          <Checkbox key={lane} label={lane} checked={column.match.lanes.includes(lane)} onChange={(on) => onChange({ ...column, match: { ...column.match, lanes: configToggleList(column.match.lanes, lane, on) } })} />
        ))}
      </Flex>
      <ConfigListInput label="Stages" hint="Comma separated. Empty means every stage." value={column.match.stages ?? []} onCommit={(stages) => onChange({ ...column, match: { ...column.match, stages } })} />
    </Flex>
  );
};

export { ConfigColumnFields };
