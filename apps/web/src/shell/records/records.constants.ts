/* @layer renderer-app @kind constants */
import { CODE_TABLE } from '../../data/registry';
import { buildRecordGrammar } from './behavior/build-record-grammar';

const GRAMMAR = buildRecordGrammar(CODE_TABLE);

const RECORD_TOKEN = GRAMMAR.token;

const RECORD_TAILS = GRAMMAR.checks;

const RECORD_PARTS = /^(?:([a-z][a-z0-9-]*)\/)?([A-Z]{1,2})-(.+)$/;

const BOARD_PATH = /^\/p\/([^?#/]+)\?(.*)$/;

const OPEN_PARAM = 'open';

const PANE_PARAM = 'pane';

const STACK_KEY = 'recordStack';

const STACK_MAX = 20;

const NO_STACK: readonly string[] = [];

const LEGACY_HASH = /^([^/]+)(?:\/([a-z][a-z-]*))?$/;

const DEP_TAB = 'links';

const ROW_CHIPS_MAX = 3;

const UNRANKED = Number.MAX_SAFE_INTEGER;

const NO_LINK_FLAG = 'This kind of record needs a link.';

export { BOARD_PATH, DEP_TAB, LEGACY_HASH, NO_LINK_FLAG, NO_STACK, OPEN_PARAM, PANE_PARAM, RECORD_PARTS, RECORD_TAILS, RECORD_TOKEN, ROW_CHIPS_MAX, STACK_KEY, STACK_MAX, UNRANKED };
