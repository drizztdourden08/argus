/* @layer renderer-app @kind component */
import { SegmentedControl } from '@drizztdourden08/tessera';
import type { LayoutToggleProps } from '../shared.type';

const LayoutToggle = <T extends string>(props: LayoutToggleProps<T>) => (
  <SegmentedControl value={props.value} options={props.options} onChange={props.onChange} selectedTone="neutral" size="sm" aria-label={props.label} />
);

export { LayoutToggle };
