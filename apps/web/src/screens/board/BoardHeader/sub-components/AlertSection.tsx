/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import type { AlertSectionProps } from '../BoardHeader.type';

const AlertSection = (props: AlertSectionProps) => {
  const { title, count, children } = props;
  if (count === 0) return null;
  return (
    <Stack gap="2xs" role="group" aria-label={title}>
      <Text variant="overline">{`${title} (${count})`}</Text>
      {children}
    </Stack>
  );
};

export { AlertSection };
