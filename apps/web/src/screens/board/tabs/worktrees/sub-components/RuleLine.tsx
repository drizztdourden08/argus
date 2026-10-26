/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { ruleDetails } from '../model';
import type { RuleLineProps } from '../worktrees-props.type';

const RuleLine = (props: RuleLineProps) => {
  const { rule, meta } = props;
  return (
    <Flex gap="xs" align="center" wrap>
      <Text.B>{rule.id}</Text.B>
      <Tag>{rule.kind}</Tag>
      <Text.Code>{rule.path}</Text.Code>
      {rule.invalid && (
        <Tag variant="urgency" color="danger" title="A rule path is the parent folder, never a wildcard. Argus ignores this rule and deletes nothing under it.">
          {`invalid: ${rule.invalid}`}
        </Tag>
      )}
      {rule.include?.length ? <Tag title="Only direct child folders whose name matches one of these">{`include ${rule.include.join(' ')}`}</Tag> : null}
      {rule.exclude?.length ? <Tag title="Direct child folders whose name matches one of these are skipped">{`exclude ${rule.exclude.join(' ')}`}</Tag> : null}
      {rule.exists === false && <Tag variant="urgency" color="danger" title="Nothing exists at this path on this machine">not found</Tag>}
      {rule.error && !rule.invalid && <Tag variant="urgency" color="danger" title={rule.error}>{rule.error}</Tag>}
      <Text variant="caption" tone="muted">{ruleDetails(rule, meta)}</Text>
      {rule.note && <Text variant="caption" tone="muted">{rule.note}</Text>}
    </Flex>
  );
};

export { RuleLine };
