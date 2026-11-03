/* @layer renderer-app @kind logic */
import type { ThreadChoice } from '../../../../../data/kanban';
import type { WithId } from '../../../../../data/react';
import { ITEM_REGISTRY } from '../../../../../data/registry';
import { categoryColor } from './category-color';
import { COORDINATOR, LEAD_PREFIX } from './look.constants';
import { OWNER, OWNER_TITLE } from './model.constants';
import type { AssigneeLook } from './model.type';

const leadLook = (key: string, tip: string): AssigneeLook => {
  const look = ITEM_REGISTRY.people.leads[key];
  return { label: `${key} lead`, tip, initials: look?.initials ?? key.slice(0, 2).toUpperCase(), color: categoryColor(look?.color), lead: key, owner: false };
};

const isLead = (assignee: string, threads: readonly WithId<ThreadChoice>[]): boolean =>
  assignee === COORDINATOR || threads.some((thread) => thread.id === assignee && thread.role === COORDINATOR);

const assigneeLook = (assignee: string, boardKey: string, threads: readonly WithId<ThreadChoice>[]): AssigneeLook => {
  if (assignee === OWNER) return { label: OWNER_TITLE, tip: 'Assigned to you', icon: 'user', owner: true };
  if (assignee.startsWith(LEAD_PREFIX)) return leadLook(assignee.slice(LEAD_PREFIX.length), `Assigned to the lead of ${assignee.slice(LEAD_PREFIX.length)}`);
  if (isLead(assignee, threads)) return leadLook(boardKey, `Assigned to ${boardKey} lead (coordinator)`);
  return { label: assignee, tip: `Assigned to ${assignee}`, initials: assignee.slice(0, 2).toUpperCase(), owner: false };
};

export { assigneeLook };
