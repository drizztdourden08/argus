/* @layer renderer-app @kind component */
import { Card, Grid } from '@drizztdourden08/tessera';
import type { TestPartProps } from '../tests-props.type';
import { DrawerSection } from './DrawerSection';
import { TextList } from './TextList';

const ExpectedLists = (props: TestPartProps) => {
  const { test } = props;
  if (test.pass.length === 0 && test.fail.length === 0) return null;
  return (
    <DrawerSection title="Expected">
      <Grid columns={2} minColWidth={224} gap="md">
        <Card title="Proves it works" tone="success" level={5}>
          <TextList lines={test.pass} label="Proves it works" />
        </Card>
        <Card title="Proves it is broken" tone="danger" level={5}>
          <TextList lines={test.fail} label="Proves it is broken" />
        </Card>
      </Grid>
    </DrawerSection>
  );
};

export { ExpectedLists };
