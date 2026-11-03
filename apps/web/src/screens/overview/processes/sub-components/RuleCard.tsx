/* @layer renderer-app @kind component */
import { Button, Inline, Tag, Text } from '@drizztdourden08/tessera';
import { asNumber, formatMemory } from '../../../../format';
import { groupCap, isOverCap, ruleTip } from '../model';
import { CALM_LOOK, OVER_LOOK } from '../processes.constants';
import type { RuleCardProps } from '../processes.type';

const RuleCard = (props: RuleCardProps) => {
  const { group, def, stale, selected, onPick, kill } = props;
  const memory = asNumber(group.memMB) ?? 0;
  const cap = groupCap(group, def);
  const over = isOverCap(memory, cap);
  const capText = over && cap !== null ? ` / cap ${formatMemory(cap)}` : '';
  const tip = ruleTip(group, cap, over);
  const canKill = stale.length > 0 && def?.killable !== false;
  return (
    <Inline gap="xs" align="center">
      <Tag
        {...(over ? OVER_LOOK : CALM_LOOK)}
        title={tip}
        selected={selected}
        selectedTone="neutral"
        onSelect={() => onPick(group.rule)}
      >
        <Inline gap="sm" align="baseline">
          <Text size="md" weight={600}>{group.rule}</Text>
          <Text variant="subtitle" size="md" numeric>{asNumber(group.count) ?? 0}</Text>
          <Text variant="subtitle" size="md" numeric>{`${formatMemory(memory)}${capText}`}</Text>
        </Inline>
      </Tag>
      {canKill && (
        <Button size="xs" variant="danger" disabled={kill.killing} onClick={() => kill.askRule(group.rule)} title="Kill the stale processes of this rule that are idle past its limit">
          {`Kill stale (${stale.length})`}
        </Button>
      )}
    </Inline>
  );
};

export { RuleCard };
