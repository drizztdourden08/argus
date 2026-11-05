/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import type { TestBodyProps } from '../OwnerFeed.type';

const TestBody = (props: TestBodyProps) => {
  const { build, handed } = props;
  if (!build) return null;
  return (
    <Flex gap="sm" align="center">
      <Tag fill={build === handed ? 'solid' : undefined}>{build === handed ? `${build} · handed to you` : build}</Tag>
    </Flex>
  );
};

export { TestBody };
