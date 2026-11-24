/* @layer renderer-app @kind component */
import { Box, Button, Callout, Flex, ScrollArea, Text } from '@drizztdourden08/tessera';
import { configAddColumn, configAddRule, configMoveColumn, configRemoveColumn, configRemoveRule, configSetColumn, configSetRule } from '../model';
import { useConfigEditor } from '../behavior/useConfigEditor';
import type { ConfigSectionProps } from '../config-editor.type';
import { ConfigColumn } from './ConfigColumn';
import { ConfigRuleCard } from './ConfigRuleCard';
import './ColumnsRulesSection.css';

const ColumnsRulesSection = (props: ConfigSectionProps) => {
  const editor = useConfigEditor(props.boardKey);
  const { config, edit, busy } = editor;
  const rules = config.rules ?? [];
  const used = new Set(config.columns.flatMap((column) => column.rules ?? []));
  return (
    <Flex direction="column" gap="md">
      <Text variant="label">Columns and rules</Text>
      <Text variant="caption" tone="muted">
        {editor.custom ? 'This board has its own columns and rules.' : 'This board runs the default columns and rules.'}
      </Text>
      {editor.problem && <Callout tone="danger" title="The config was not saved">{editor.problem}</Callout>}
      <ScrollArea axis="x" scrollbar="slim">
        <Box className="config-preview__strip">
          {config.columns.map((column, index) => (
            <Box key={index} className="config-preview__column">
              <ConfigColumn
                column={column}
                index={index}
                count={config.columns.length}
                rules={rules}
                onChange={(next) => edit((current) => configSetColumn(current, index, next))}
                onMove={(step) => edit((current) => configMoveColumn(current, index, step))}
                onRemove={() => edit((current) => configRemoveColumn(current, index))}
                onRuleChange={(at, next) => edit((current) => configSetRule(current, at, next))}
                onRuleRemove={(at) => edit((current) => configRemoveRule(current, at))}
              />
            </Box>
          ))}
          <Button size="sm" variant="secondary" onClick={() => edit(configAddColumn)}>Add a column</Button>
        </Box>
      </ScrollArea>
      <Text variant="label">Rules not on a column</Text>
      <Flex gap="md" wrap align="start">
        {rules.map((rule, index) => !used.has(rule.id) && (
          <Box key={index} className="config-preview__column">
            <ConfigRuleCard
              rule={rule}
              onChange={(next) => edit((current) => configSetRule(current, index, next))}
              onRemove={() => edit((current) => configRemoveRule(current, index))}
            />
          </Box>
        ))}
      </Flex>
      <Flex>
        <Button size="sm" variant="secondary" onClick={() => edit(configAddRule)}>Add a rule</Button>
      </Flex>
      <Flex gap="sm">
        <Button size="sm" variant="primary" disabled={busy || !editor.dirty} onClick={() => void editor.save()}>Save the columns and rules</Button>
        <Button size="sm" variant="ghost" disabled={busy || !editor.dirty} onClick={editor.discard}>Discard changes</Button>
        <Button size="sm" variant="secondary" disabled={busy} onClick={() => void editor.useDefault()}>Use the default columns and rules</Button>
      </Flex>
    </Flex>
  );
};

export { ColumnsRulesSection };
