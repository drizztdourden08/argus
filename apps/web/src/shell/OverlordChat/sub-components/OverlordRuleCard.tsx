/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { RuleCard } from '../../RuleCard';
import { nextRunText } from '../behavior/next-run-text';
import type { OverlordRuleCardProps } from '../behavior/chat-panels.type';

const OverlordRuleCard = (props: OverlordRuleCardProps) => {
  const { rule, busy, onToggle, onEdit, onRun, onDelete } = props;
  return (
    <RuleCard
      title={rule.title}
      kind="schedule"
      badge={rule.createdBy}
      facts={[{ label: 'Every', value: `${rule.everyMin} min` }]}
      reaction={`Next run ${nextRunText(rule, Date.now())}.`}
      toggle={{ label: 'Enabled', checked: rule.enabled, disabled: busy, onChange: onToggle }}
      actions={<Button size="xs" disabled={busy} onClick={onRun}>Run now</Button>}
      editLabel={`Edit ${rule.title}`}
      removeLabel={`Delete ${rule.title}`}
      busy={busy}
      onEdit={onEdit}
      onRemove={onDelete}
    />
  );
};

export { OverlordRuleCard };
