/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { plainText } from '../../../plain-text';
import { REMOVED_SHOWN } from '../model';
import type { RemovedSectionProps } from '../worktrees-props.type';

const RemovedSection = (props: RemovedSectionProps) => {
  const { removed } = props;
  if (removed.length === 0) return null;
  return (
    <Flex direction="column" gap="xs" role="group" aria-label="Worktrees Argus removed by itself">
      <Text variant="caption" tone="muted">Removed by the worktree hygiene: the released worktrees, newest first.</Text>
      {removed.slice(0, REMOVED_SHOWN).map((done) => (
        <Flex key={`${done.id}-${done.at}`} gap="xs" align="center" wrap>
          <Tag title={plainText(done.text)}>removed</Tag>
          <Text.B>{done.path}</Text.B>
          <Text variant="caption">
            {done.thread ? `${done.thread} · ` : ''}
            <Age value={done.at} />
            {' ago'}
          </Text>
        </Flex>
      ))}
    </Flex>
  );
};

export { RemovedSection };
