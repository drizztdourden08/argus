/* @layer renderer-app @kind logic */
import type { TabLink } from '../../hub/tab-link';
import { useStreams } from './useStreams';

const useLink = (): TabLink => useStreams().link;

export { useLink };
