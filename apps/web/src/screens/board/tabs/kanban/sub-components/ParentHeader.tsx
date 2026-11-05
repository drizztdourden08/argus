/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordLink, TypeIcon } from '../../../../../shell';
import type { ParentHeaderProps } from '../kanban-props.type';
import { DEFAULT_TYPE } from '../model';

const ParentHeader = (props: ParentHeaderProps) => {
  const { parent, withCode } = props;
  return (
    <Flex as="span" gap="sm" align="center">
      <TypeIcon kind={parent.type ?? DEFAULT_TYPE} />
      {withCode ? <RecordLink id={parent.id} /> : null}
      <Text as="span" variant="subtitle" leading="16" clamp={1}>{parent.title}</Text>
    </Flex>
  );
};

export { ParentHeader };
