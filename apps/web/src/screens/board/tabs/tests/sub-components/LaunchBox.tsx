/* @layer renderer-app @kind component */
import { Callout, DateText, Disclosure, Flex, Markdown, Text } from '@drizztdourden08/tessera';
import { NO_BUNDLE } from '../model';
import type { LaunchBoxProps } from '../tests-props.type';
import { HandButton } from './HandButton';
import { LaunchCommand } from './LaunchCommand';

const LaunchBox = (props: LaunchBoxProps) => {
  const { view, boardKey, runs, state } = props;
  const { group } = view;
  if (group.tag === NO_BUNDLE) return null;
  const draft = state === 'draft';
  const { build } = group;
  return (
    <Flex direction="column" gap="xs" aria-label={`Launch ${group.tag}`} role="group">
      {draft && (
        <Callout tone="info">
          Draft: not built yet, so no Play and it cannot be handed to you. The lead sets coord build ready {group.tag} when it is built and testable.
        </Callout>
      )}
      <LaunchCommand view={view} boardKey={boardKey} runs={runs} draft={draft} />
      {build?.note && <Disclosure size="sm" summary="Note"><Markdown size="sm">{build.note}</Markdown></Disclosure>}
      <Flex gap="sm" align="center" wrap>
        {(!draft || group.handed) && <HandButton boardKey={boardKey} tag={group.tag} handed={group.handed} />}
        {build?.updatedAt && <Text variant="caption" tone="muted">updated <DateText value={build.updatedAt} format="relative" /></Text>}
      </Flex>
    </Flex>
  );
};

export { LaunchBox };
