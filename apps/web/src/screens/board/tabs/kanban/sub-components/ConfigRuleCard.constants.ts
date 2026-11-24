/* @layer renderer-app @kind constants */
import { RULE_KINDS } from '../model';

const KIND_OPTIONS = RULE_KINDS.map((entry) => ({ value: entry.kind, label: entry.kind }));

export { KIND_OPTIONS };
