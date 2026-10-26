/* @layer renderer-app @kind component */
import { Disclosure, Flex, Text } from '@drizztdourden08/tessera';
import type { RulesSectionProps } from '../worktrees-props.type';
import { RuleLine } from './RuleLine';

const RulesSection = (props: RulesSectionProps) => {
  const { rules, meta, open, onOpen } = props;
  if (rules.length === 0) return null;
  return (
    <Disclosure open={open} onOpenChange={onOpen} size="sm" summary={`Rules (${rules.length})`}>
      <Flex direction="column" gap="xs">
        <Text variant="caption" tone="muted">The registered thread adds these with <Text.Code>coord worktree rule add</Text.Code>.</Text>
        {rules.map((rule) => <RuleLine key={rule.id} rule={rule} meta={meta} />)}
      </Flex>
    </Disclosure>
  );
};

export { RulesSection };
