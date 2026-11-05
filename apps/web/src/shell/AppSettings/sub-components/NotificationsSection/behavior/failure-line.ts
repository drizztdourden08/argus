/* @layer renderer-app @kind logic */
import { errorMessage } from '../../../../../data/api';

const failureLine = (error: unknown): string => `The test failed: ${errorMessage(error)}`;

export { failureLine };
