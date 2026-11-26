/* @layer renderer-app @kind component */
import { Icon, Inline, Stack, Text } from '@drizztdourden08/tessera';
import { whenText } from '../model';
import type { NextArrowProps } from '../next.type';
import { NextRun } from './NextRun';

const NextArrow = (props: NextArrowProps) => (
  <Stack gap="xs" align="center">
    <Inline gap="xs" align="center">
      <NextRun phase={props.phase} />
      <Icon name="arrow-right" />
    </Inline>
    <Text variant="caption" tone="muted">{whenText(props.when)}</Text>
  </Stack>
);

export { NextArrow };
