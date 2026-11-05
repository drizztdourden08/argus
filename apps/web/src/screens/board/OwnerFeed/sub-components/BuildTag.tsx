/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import type { BuildTagProps } from '../OwnerFeed.type';

const BuildTag = (props: BuildTagProps) => {
  const { build } = props;
  if (!build) return null;
  return (
    <Flex gap="sm" align="center">
      <Tag>{build}</Tag>
    </Flex>
  );
};

export { BuildTag };
