/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { RecordMenuProps } from '../records.type';
import { RecordMenu } from './RecordMenu';
import './MenuSlot.css';

const MenuSlot = (props: RecordMenuProps) => (props.groups.length === 0 ? <Box className="menu-slot" aria-hidden /> : <RecordMenu {...props} />);

export { MenuSlot };
