/* @layer renderer-app @kind component */
import { Card, List, ListItem, Small, Stack, Tag, Text } from '@drizztdourden08/tessera';
import { stallsOf, useStalls } from '../../../../data/stalls';
import { Age, BAD_LOOK, CALM_LOOK } from '../../../../shell';
import { ProjectLink } from '../../projects';

const StallList = () => {
  const stalls = stallsOf(useStalls().docs);
  if (stalls.length === 0) return null;
  return (
    <Card>
      <Stack gap="xs">
        <Text variant="overline">Stalls</Text>
        <List aria-label="Stalls">
          {stalls.map((stall) => (
            <ListItem key={stall.id} bare>
              <Small tone="muted"><Age value={stall.since} /></Small>
              <Tag pill {...BAD_LOOK}>{stall.kind}</Tag>
              <ProjectLink project={stall.project} />
              <Small title={stall.text}>{stall.text}</Small>
              {stall.ackedAt ? <Tag pill {...CALM_LOOK} title={stall.ackNote ?? undefined}>acked</Tag> : null}
            </ListItem>
          ))}
        </List>
      </Stack>
    </Card>
  );
};

export { StallList };
