/* @layer renderer-app @kind logic */
import { RULE_KINDS } from './config-editor.constants';
import type { RuleKind } from './config-editor.type';

const configKind = (kind: string): RuleKind => RULE_KINDS.find((entry) => entry.kind === kind) ?? RULE_KINDS[0];

export { configKind };
