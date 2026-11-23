/* @layer renderer-app @kind logic */
import { FIRST_KIND, RULE_KINDS } from './config-editor.constants';
import type { RuleKind } from './config-editor.type';

const configKind = (kind: string): RuleKind => RULE_KINDS.find((entry) => entry.kind === kind) ?? FIRST_KIND;

export { configKind };
