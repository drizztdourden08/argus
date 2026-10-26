/* @layer root-config @kind config */
import { defineWorkspace } from '@drizztdourden08/brock-build';

export default defineWorkspace({ name: 'argus', gate: { scripts: ['lint', 'test'] } });
