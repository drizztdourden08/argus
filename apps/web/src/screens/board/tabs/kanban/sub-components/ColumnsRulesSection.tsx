/* @layer renderer-app @kind component */
import { Button, Callout, Flex, Text } from '@drizztdourden08/tessera';
import { configAddColumn, configAddRule, configMoveColumn, configRemoveColumn, configRemoveRule, configSetColumn, configSetRule } from '../model';
import { useConfigEditor } from '../behavior/useConfigEditor';
import type { ConfigSectionProps } from '../config-editor.type';
import { ConfigColumnRow } from './ConfigColumnRow';
import { ConfigRuleRow } from './ConfigRuleRow';

const ColumnsRulesSection = (props: ConfigSectionProps) => {
  const editor = useConfigEditor(props.boardKey);
  const { config, edit, busy } = editor;
  const rules = config.rules ?? [];
  return (
    <Flex direction="column" gap="md">
      <Text variant="label">Columns and rules</Text>
      <Text variant="caption" tone="muted">
        {editor.custom ? 'This board has its own columns and rules.' : 'This board runs the default columns and rules.'}
      </Text>
      {editor.problem && <Callout tone="danger" title="The config was not saved">{editor.problem}</Callout>}
      <Text variant="label">Columns</Text>
      {config.columns.map((column, index) => (
        <ConfigColumnRow
          key={index}
          column={column}
          index={index}
          count={config.columns.length}
          rules={rules}
          onChange={(next) => edit((current) => configSetColumn(current, index, next))}
          onMove={(step) => edit((current) => configMoveColumn(current, index, step))}
          onRemove={() => edit((current) => configRemoveColumn(current, index))}
        />
      ))}
      <Button size="sm" variant="secondary" onClick={() => edit(configAddColumn)}>Add a column</Button>
      <Text variant="label">Rules</Text>
      {rules.map((rule, index) => (
        <ConfigRuleRow
          key={index}
          rule={rule}
          onChange={(next) => edit((current) => configSetRule(current, index, next))}
          onRemove={() => edit((current) => configRemoveRule(current, index))}
        />
      ))}
      <Button size="sm" variant="secondary" onClick={() => edit(configAddRule)}>Add a rule</Button>
      <Flex gap="sm">
        <Button size="sm" variant="primary" disabled={busy || !editor.dirty} onClick={() => void editor.save()}>Save the columns and rules</Button>
        <Button size="sm" variant="ghost" disabled={busy || !editor.dirty} onClick={editor.discard}>Discard changes</Button>
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void editor.useDefault()}>Use the default</Button>
      </Flex>
    </Flex>
  );
};

export { ColumnsRulesSection };
