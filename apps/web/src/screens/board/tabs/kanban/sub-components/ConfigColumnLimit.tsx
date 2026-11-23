/* @layer renderer-app @kind component */
import { Field, Flex, NumberInput, Select, Toggle } from '@drizztdourden08/tessera';
import { LIMIT_MODES } from '../model';
import type { KanbanColumnLimit } from '../../../../../data/types';
import type { ConfigColumnLimitProps } from '../config-editor.type';

const FIRST_LIMIT: KanbanColumnLimit = { max: 3, mode: 'warn' };

const MODE_OPTIONS = LIMIT_MODES.map((mode) => ({ value: mode, label: mode }));

const ConfigColumnLimit = (props: ConfigColumnLimitProps) => {
  const { column, onChange } = props;
  const limit = column.limit ?? null;
  const setLimit = (next: KanbanColumnLimit | null) => onChange({ ...column, limit: next });
  return (
    <Flex direction="column" gap="sm">
      <Toggle label="Limit the cards" checked={limit !== null} onChange={(on) => setLimit(on ? FIRST_LIMIT : null)} />
      {limit && (
        <Flex gap="md" align="end">
          <Field label="Most cards" width="sm">
            <NumberInput min={1} step={1} value={limit.max} onChange={(max) => setLimit({ ...limit, max: Math.max(1, Math.round(max)) })} />
          </Field>
          <Field label="Over the limit" width="sm">
            <Select options={MODE_OPTIONS} value={limit.mode} onChange={(mode) => setLimit({ ...limit, mode: mode as KanbanColumnLimit['mode'] })} />
          </Field>
        </Flex>
      )}
    </Flex>
  );
};

export { ConfigColumnLimit };
