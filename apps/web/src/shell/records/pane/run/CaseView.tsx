/* @layer renderer-app @kind component */
import { Flex, Pressable, Text } from '@drizztdourden08/tessera';
import { CaseLines } from './CaseLines';
import { CaseLinks } from './CaseLinks';
import type { CaseViewProps } from './run.type';

const CaseView = (props: CaseViewProps) => {
  const { id, test, open } = props;
  if (!test) return <Text tone="muted">{id} is not on this board, so it cannot be run here.</Text>;
  return (
    <Flex direction="column" gap="md" aria-label={`Case ${id}`} role="group">
      <Flex gap="sm" align="baseline" wrap>
        <Pressable aria-label={`Open ${id}`} onClick={() => open(id)}>
          <Text as="span" variant="caption" weight={600} mono>{id}</Text>
        </Pressable>
        <Text as="span" weight={600} smartTimes>{test.title}</Text>
      </Flex>
      <CaseLines title="Setup" lines={(test.setup ?? []).map((entry) => `${entry.label}: ${entry.text}`)} />
      <CaseLines title="Steps" lines={test.steps ?? []} ordered />
      <CaseLines title="Pass when" lines={test.pass ?? []} />
      <CaseLines title="Fail when" lines={test.fail ?? []} />
      <CaseLines title="Notes" lines={test.notes ?? []} />
      <CaseLinks test={test} open={open} />
    </Flex>
  );
};

export { CaseView };
