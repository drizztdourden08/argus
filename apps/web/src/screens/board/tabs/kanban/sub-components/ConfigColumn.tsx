/* @layer renderer-app @kind component */
import { Card, Divider, Flex, Icon, IconButton, Tag } from '@drizztdourden08/tessera';
import type { ConfigColumnProps } from '../config-editor.type';
import { configAccent, rulesText } from '../model';
import { ConfigColumnFields } from './ConfigColumnFields';
import { ConfigColumnLimit } from './ConfigColumnLimit';
import { ConfigColumnRules } from './ConfigColumnRules';

const ConfigColumn = (props: ConfigColumnProps) => {
  const { column, index, count, rules, onChange, onMove, onRemove, onRuleChange, onRuleRemove } = props;
  const name = column.name || column.id;
  const accent = configAccent(column.accent);
  return (
    <Card
      title={name}
      subtitle={rulesText((column.rules ?? []).length)}
      aria-label={`Column ${name}`}
      actions={(
        <Flex gap="xs" align="center">
          {accent && <Tag variant="category" color={accent}>{accent}</Tag>}
          <IconButton label={`Move ${name} left`} variant="ghost" size="sm" disabled={index === 0} onClick={() => onMove(-1)}><Icon name="chevron-left" /></IconButton>
          <IconButton label={`Move ${name} right`} variant="ghost" size="sm" disabled={index === count - 1} onClick={() => onMove(1)}><Icon name="chevron-right" /></IconButton>
          <IconButton label={`Remove ${name}`} variant="ghost" size="sm" onClick={onRemove}><Icon name="trash-2" /></IconButton>
        </Flex>
      )}
    >
      <Flex direction="column" gap="md">
        <ConfigColumnFields column={column} onChange={onChange} />
        <ConfigColumnLimit column={column} onChange={onChange} />
        <Divider />
        <ConfigColumnRules column={column} rules={rules} onChange={onChange} onRuleChange={onRuleChange} onRuleRemove={onRuleRemove} />
      </Flex>
    </Card>
  );
};

export { ConfigColumn };
