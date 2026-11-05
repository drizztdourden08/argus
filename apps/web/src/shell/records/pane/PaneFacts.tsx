/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../format';
import { useOwnerProfile } from '../../../owner';
import { FactSelect, threadChoices } from '../../../screens/board/shared';
import { NO_PARENT, assigneeOf } from '../../../screens/board/tabs/kanban/model';
import { BlockerLine } from '../../../screens/board/tabs/kanban/sub-components/BlockerLine';
import { parentChoices } from './model/parent-choices';
import { paneType } from './model/pane-type';
import { peopleOptions } from './model/people-options';
import { stageFact } from './model/stage-fact';
import type { PaneFactsProps } from './pane.type';

const PaneFacts = (props: PaneFactsProps) => {
  const { item, items, threads, blocking, now, ops } = props.data;
  const owner = useOwnerProfile();
  const stage = stageFact(item);
  const parents = parentChoices(item, items);
  const assignee = assigneeOf(item);
  return (
    <Flex direction="column" gap="sm">
      <Flex gap="md" align="center" wrap>
        <FactSelect label={stage.label} options={stage.options} value={stage.value} onPick={(next) => void ops.move(item, stage.target(next), `Moved ${item.id}.`)} />
        <FactSelect label="Thread" options={threadChoices(threads, item.thread)} value={item.thread} onPick={(thread) => void ops.move(item, { thread }, `Moved ${item.id}.`)} />
        <FactSelect label="Assignee" options={peopleOptions(threads, assignee, owner)} value={assignee} onPick={(next) => void ops.setField(item, 'assignee', next, `Assigned ${item.id}.`)} />
        {parents.length > 0 && (
          <FactSelect
            label="Parent"
            options={parents}
            value={item.parent ?? NO_PARENT}
            onPick={(next) => void ops.setField(item, 'parent', next === NO_PARENT ? null : next, `Moved ${item.id} under ${next}.`)}
          />
        )}
      </Flex>
      {item.blocker && <BlockerLine blocker={item.blocker} link={blocking} />}
      <Text variant="caption" tone="muted">{`${paneType(item)?.label ?? 'Item'}, rev${item.rev ?? 1}, updated by ${item.updatedBy ?? item.createdBy ?? 'unknown'} ${formatAgo(item.updatedAt, now)} ago`}</Text>
    </Flex>
  );
};

export { PaneFacts };
