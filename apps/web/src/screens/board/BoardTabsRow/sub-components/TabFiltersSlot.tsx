/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { filtersSlot } from '../behavior/slot-store';

const TabFiltersSlot = () => <Box ref={filtersSlot.set} />;

export { TabFiltersSlot };
