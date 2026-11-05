/* @layer renderer-app @kind component */
import { SegmentedControl } from '@drizztdourden08/tessera';
import { LAYOUT_CHOICES } from '../tests.constants';
import type { LayoutControlProps } from '../tests-head-props.type';

const LayoutControl = (props: LayoutControlProps) => (
  <SegmentedControl value={props.layout} options={LAYOUT_CHOICES} onChange={props.onChange} selectedTone="neutral" size="sm" aria-label="Case view" />
);

export { LayoutControl };
