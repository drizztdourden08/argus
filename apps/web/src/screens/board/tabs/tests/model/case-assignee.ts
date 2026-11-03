/* @layer renderer-app @kind logic */
import { OWNER_ASSIGNEE } from './model.constants';
import type { TestDoc } from './model.type';

const caseAssignee = (test: TestDoc): string => (test.mode === 'manual' ? OWNER_ASSIGNEE : test.thread ?? '');

export { caseAssignee };
