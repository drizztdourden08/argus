/* @layer renderer-app @kind component */
import { DateText, Flex, List, ListItem, Markdown, Status, Text } from '@drizztdourden08/tessera';
import { byLabel } from '../model';
import { TEST_STATUSES } from '../tests.constants';
import type { ResultHistoryProps } from '../tests-props.type';
import { DrawerSection } from './DrawerSection';

const ResultHistory = (props: ResultHistoryProps) => {
  const { runs } = props;
  if (runs.length < 2) return null;
  return (
    <DrawerSection title="Results history" count={runs.length}>
      <List aria-label="Results history">
        {[...runs].reverse().map((run) => (
          <ListItem key={`${run.at}-${run.status}`}>
            <Flex gap="sm" align="baseline" wrap>
              <Status map={TEST_STATUSES} value={run.status as keyof typeof TEST_STATUSES} fallback="moved" dot />
              <Text variant="caption" tone="muted"><DateText value={run.at} format="relative" /> by {byLabel(run.by)}</Text>
              <Markdown size="sm">{run.text}</Markdown>
            </Flex>
          </ListItem>
        ))}
      </List>
    </DrawerSection>
  );
};

export { ResultHistory };
