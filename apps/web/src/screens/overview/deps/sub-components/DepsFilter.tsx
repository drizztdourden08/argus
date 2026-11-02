/* @layer renderer-app @kind component */
import { SegmentedControl } from '@drizztdourden08/tessera';
import { DEP_FILTERS } from '../deps.constants';
import type { DepsFilterProps } from '../deps.type';
import type { DepFilter } from '../model';

const DepsFilter = (props: DepsFilterProps) => {
  const { value, onChange } = props;
  return <SegmentedControl<DepFilter> selectedTone="neutral" aria-label="Show dependencies" size="sm" value={value} options={DEP_FILTERS} onChange={onChange} />;
};

export { DepsFilter };
