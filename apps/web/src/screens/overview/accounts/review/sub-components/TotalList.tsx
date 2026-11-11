/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import type { TotalListProps } from '../review-props.type';

const TotalList = (props: TotalListProps) => {
  const { title, lines } = props;
  if (lines.length === 0) return null;
  return (
    <Stack gap="xs">
      <Text variant="overline">{title}</Text>
      {lines.map((line) => <Text key={line.label}>{`${line.count} x ${line.label}`}</Text>)}
    </Stack>
  );
};

export { TotalList };
