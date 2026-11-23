/* @layer renderer-app @kind component */
import { Disclosure, Flex, Icon, IconButton, Text } from '@drizztdourden08/tessera';
import type { ConfigColumnRowProps } from '../config-editor.type';
import { ConfigColumnFields } from './ConfigColumnFields';
import { ConfigColumnLimit } from './ConfigColumnLimit';

const summaryText = (props: ConfigColumnRowProps): string => {
  const { column } = props;
  const limit = column.limit ? `, max ${column.limit.max} ${column.limit.mode}` : '';
  return `${column.match.lanes.join(' + ') || 'no lane'}${limit}, ${(column.rules ?? []).length} rules`;
};

const ConfigColumnRow = (props: ConfigColumnRowProps) => {
  const { column, index, count, rules, onChange, onMove, onRemove } = props;
  const name = column.name || column.id;
  return (
    <Disclosure
      size="sm"
      summary={<Flex gap="sm" align="center"><Text variant="label">{name}</Text><Text variant="caption" tone="muted">{summaryText(props)}</Text></Flex>}
      end={(
        <Flex gap="xs">
          <IconButton label={`Move ${name} up`} variant="ghost" size="sm" disabled={index === 0} onClick={() => onMove(-1)}><Icon name="chevron-up" /></IconButton>
          <IconButton label={`Move ${name} down`} variant="ghost" size="sm" disabled={index === count - 1} onClick={() => onMove(1)}><Icon name="chevron-down" /></IconButton>
          <IconButton label={`Remove ${name}`} variant="ghost" size="sm" onClick={onRemove}><Icon name="trash" /></IconButton>
        </Flex>
      )}
    >
      <Flex direction="column" gap="md">
        <ConfigColumnFields column={column} rules={rules} onChange={onChange} />
        <ConfigColumnLimit column={column} onChange={onChange} />
      </Flex>
    </Disclosure>
  );
};

export { ConfigColumnRow };
