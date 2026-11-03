/* @layer renderer-app @kind constants */
import { ITEM_STORED } from '../../../data/registry';

const ITEM_LETTER = ITEM_STORED;

const ITEM_FALLBACK_KIND = 'task';

const TAIL_NUMBER = '\\d+';

const TAIL_VERSION = '\\d+(?:\\.\\d+)*(?:-[a-z0-9.]*[a-z0-9])?';

const TAIL_NAME = '[a-z0-9](?:[a-z0-9.@-]*[a-z0-9])?';

const BOARD_PREFIX = '(?:[a-z][a-z0-9-]*/)?';

export { BOARD_PREFIX, ITEM_FALLBACK_KIND, ITEM_LETTER, TAIL_NAME, TAIL_NUMBER, TAIL_VERSION };
