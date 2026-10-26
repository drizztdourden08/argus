/* @layer renderer-app @kind logic */
import { PARENT_SEGMENT, QUERY_OR_HASH } from '../markdown.constants';

const leavesFolder = (address: string): boolean => PARENT_SEGMENT.test(address.replace(QUERY_OR_HASH, ''));

export { leavesFolder };
