/* @layer renderer-app @kind logic */
import type { MarkdownLinkId } from '@drizztdourden08/tessera';
import { BOARD_ID } from '../markdown.constants';

const boardIdRule = (boardKey: string): MarkdownLinkId => ({ pattern: BOARD_ID, href: (id) => `/p/${boardKey}#${id}` });

export { boardIdRule };
