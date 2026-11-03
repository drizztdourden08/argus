/* @layer renderer-app @kind component */
import { Button, Flex, Tag } from '@drizztdourden08/tessera';
import { useRecordPane } from '../../../../shell';
import type { TestBodyProps } from '../OwnerFeed.type';

const TestBody = (props: TestBodyProps) => {
  const { facts, build, handed } = props;
  const pane = useRecordPane();
  return (
    <Flex gap="sm" align="center">
      {build ? <Tag fill={build === handed ? 'solid' : undefined}>{build === handed ? `${build} · handed to you` : build}</Tag> : null}
      <Button size="sm" variant="tertiary" onClick={() => pane.open(facts.code, facts.tab)}>Open</Button>
    </Flex>
  );
};

export { TestBody };
