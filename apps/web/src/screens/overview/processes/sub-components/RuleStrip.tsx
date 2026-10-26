/* @layer renderer-app @kind component */
import { Flex, StackedBar, Stack } from '@drizztdourden08/tessera';
import { asNumber, formatMemory } from '../../../../format';
import { staleOf } from '../model';
import type { RuleStripProps } from '../processes.type';
import { KillConfirm } from './KillConfirm';
import { RuleCard } from './RuleCard';

const RuleStrip = (props: RuleStripProps) => {
  const { groups, rules, procs, rule, onPick, kill } = props;
  if (groups.length === 0) return null;
  const asking = kill.ask?.kind === 'rule' ? kill.ask.rule : null;
  const stale = asking ? staleOf(procs, asking) : [];
  const staleMemory = stale.reduce((sum, proc) => sum + (asNumber(proc.memMB) ?? 0), 0);
  const segments = groups.map((group) => ({ id: group.rule, label: group.rule, value: asNumber(group.memMB) ?? 0 }));
  return (
    <Stack gap="sm">
      <StackedBar label="Memory by rule" segments={segments} format={formatMemory} />
      <Flex gap="sm" wrap>
        {groups.map((group) => (
          <RuleCard
            key={group.rule}
            group={group}
            def={rules.find((def) => def.name === group.rule)}
            stale={staleOf(procs, group.rule)}
            selected={rule === group.rule}
            onPick={onPick}
            kill={kill}
          />
        ))}
      </Flex>
      {asking && (
        <KillConfirm
          question={`Kill ${stale.length} stale ${asking} process${stale.length === 1 ? '' : 'es'} (${formatMemory(staleMemory)})?`}
          busy={kill.killing}
          onKill={() => kill.killStale(asking)}
          onCancel={kill.cancel}
        />
      )}
    </Stack>
  );
};

export { RuleStrip };
