/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import type { RunBodyProps } from '../OwnerFeed.type';

const RunBody = (props: RunBodyProps) => {
  const { run } = props;
  return (
    <Flex gap="sm" align="center">
      {run.build ? <Tag>{run.build}</Tag> : null}
      <Tag color="neutral">{`${run.cases} ${run.cases === 1 ? 'case' : 'cases'}`}</Tag>
    </Flex>
  );
};

export { RunBody };
