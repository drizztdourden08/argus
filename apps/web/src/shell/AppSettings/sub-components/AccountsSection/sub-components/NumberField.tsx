/* @layer renderer-app @kind component */
import { NumberInput, Stack, Text } from '@drizztdourden08/tessera';
import type { NumberFieldProps } from '../AccountsSection.type';

const NumberField = (props: NumberFieldProps) => (
  <Stack gap="xs">
    <Text variant="caption" tone="muted">{props.label}</Text>
    <NumberInput size="sm" aria-label={props.label} step={props.step} min={props.min} max={props.max} value={props.value} onChange={props.onChange} />
  </Stack>
);

export { NumberField };
