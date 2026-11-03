/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { RecordLink } from '../../../../../../shell';
import type { RefTagsProps } from '../pane.type';

const RefTags = (props: RefTagsProps) => {
  const { ids } = props;
  return (
    <Flex gap="xs" wrap>
      {ids.map((id) => (
        <RecordLink key={id} id={id}><Tag color="neutral">{id}</Tag></RecordLink>
      ))}
    </Flex>
  );
};

export { RefTags };
