/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { filtersSlot } from '../behavior/filters-slot';

const TabFiltersSlot = () => <Box ref={filtersSlot.set} />;

export { TabFiltersSlot };
