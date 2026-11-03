/* @layer renderer-app @kind component */
import { Card, Flex, Stack, Tag, Text } from '@drizztdourden08/tessera';
import { stallsOf, useStalls } from '../../../../data/stalls';
import { Age, CALM_LOOK } from '../../../../shell';
import { ProjectLink } from '../../projects';
import { leadOf, OverviewList, OverviewRow, recordOfText } from '../../records';

const StallList = () => {
  const stalls = stallsOf(useStalls().docs);
  if (stalls.length === 0) return null;
  return (
    <Card>
      <Stack gap="xs">
        <Text variant="overline">Stalls</Text>
        <OverviewList label="Stalls">
          {stalls.map((stall) => (
            <OverviewRow
              key={stall.id}
              target={recordOfText(`${stall.subject} ${stall.text}`, stall.project, stall.kind)}
              assignee={leadOf(stall.project)}
              title={stall.text}
              chips={[{ id: 'kind', text: stall.kind, title: `Stalled: ${stall.kind}` }]}
              trailing={
                <Flex gap="xs" align="center">
                  {stall.ackedAt ? <Tag pill {...CALM_LOOK} title={stall.ackNote ?? undefined}>acked</Tag> : null}
                  <ProjectLink project={stall.project} />
                  <Text variant="caption" tone="muted"><Age value={stall.since} /></Text>
                </Flex>
              }
            />
          ))}
        </OverviewList>
      </Stack>
    </Card>
  );
};

export { StallList };
