/* @layer renderer-app @kind logic */
import type { RegistryKind } from '../../../data/registry';
import { FIXED_LEAD, FIXED_OWNER } from './assignee-of-record.constants';
import { LEAD_PREFIX, OWNER } from './look.constants';

const textOf = (value: unknown): string => (typeof value === 'string' ? value : '');

const projectOf = (value: unknown): string => {
  if (typeof value !== 'object' || value === null) return '';
  return textOf(Reflect.get(value, 'project'));
};

const assigneeOfRecord = (kind: RegistryKind, doc: Readonly<Record<string, unknown>>, boardKey: string): string => {
  if (kind.assignee === FIXED_OWNER) return OWNER;
  if (kind.assignee === FIXED_LEAD) return `${LEAD_PREFIX}${boardKey}`;
  const value = doc[kind.assignee];
  const project = projectOf(value);
  if (project) return `${LEAD_PREFIX}${project}`;
  return textOf(value) || textOf(doc.thread);
};

export { assigneeOfRecord };
