/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { OPENABLE_REF } from '../model/pane-model.constants';
import type { RefTagsProps } from '../pane.type';

const RefTags = (props: RefTagsProps) => {
  const { ids, open } = props;
  return (
    <Flex gap="xs" wrap>
      {ids.map((id) =>
        OPENABLE_REF.test(id) ? (
          <Tag key={id} color="neutral" title={`Open ${id}`} onClick={() => open(id)}>{id}</Tag>
        ) : (
          <Tag key={id} color="neutral">{id}</Tag>
        ),
      )}
    </Flex>
  );
};

export { RefTags };
