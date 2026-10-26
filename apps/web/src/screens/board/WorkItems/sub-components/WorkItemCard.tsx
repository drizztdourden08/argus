/* @layer renderer-app @kind component */
import { Disclosure, Flex, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../shell';
import type { WorkItemCardProps } from '../WorkItems.type';
import { CriteriaList } from './CriteriaList';
import { ItemHeader } from './ItemHeader';
import { ItemNoteForm } from './ItemNoteForm';

const WorkItemCard = (props: WorkItemCardProps) => {
  const { item, need, poster } = props;
  const criteria = item.ac ?? [];
  return (
    <Disclosure summary={<ItemHeader item={item} need={need} />}>
      <Flex direction="column" gap="sm">
        {item.description ? <BoardMarkdown size="sm" source={item.description} /> : null}
        {criteria.length > 0 ? (
          <Flex direction="column" gap="xs">
            <Text variant="overline" tone="muted">Acceptance criteria · {criteria.filter((c) => c.done).length}/{criteria.length}</Text>
            <CriteriaList criteria={criteria} label={`Acceptance criteria of ${item.id}`} />
          </Flex>
        ) : null}
        <ItemNoteForm item={item} poster={poster} />
      </Flex>
    </Disclosure>
  );
};

export { WorkItemCard };
