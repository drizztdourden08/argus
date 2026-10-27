/* @layer renderer-app @kind component */
import { Badge, Divider, Flex, Markdown } from '@drizztdourden08/tessera';
import type { StepListProps } from '../tests-drawer-props.type';

const StepList = (props: StepListProps) => {
  const { lines } = props;
  return (
    <Flex direction="column" gap="sm" role="list" aria-label="Steps">
      {lines.map((line, index) => (
        <Flex key={`${index}-${line}`} direction="column" gap="sm" role="listitem">
          {index > 0 && <Divider />}
          <Flex gap="sm" align="center">
            <Badge variant="inline" value={index + 1} color="tame" />
            <Markdown size="sm">{line}</Markdown>
          </Flex>
        </Flex>
      ))}
    </Flex>
  );
};

export { StepList };
