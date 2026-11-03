/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { RecordLink, ShownCode } from '../../..';
import type { RefTagsProps } from './fields.type';

const RefTags = (props: RefTagsProps) => {
  const { ids } = props;
  return (
    <Flex gap="xs" wrap>
      {ids.map((id) => (
        <RecordLink key={id} id={id}><Tag color="neutral"><ShownCode id={id} /></Tag></RecordLink>
      ))}
    </Flex>
  );
};

export { RefTags };
