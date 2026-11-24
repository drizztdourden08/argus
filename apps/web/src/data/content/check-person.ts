/* @layer renderer-app @kind logic */
import { knowsLead } from '../registry';
import type { ItemField } from '../registry';
import { canonicalThread } from './canonical-thread';
import { LEAD_PREFIX, PERSON_PATTERN } from './content.constants';
import type { ContentContext, Verdict } from './content.type';
import { reject } from './reject';

const checkLead = (field: string, spec: ItemField, text: string): Verdict => {
  if (spec.leads && knowsLead(text.slice(LEAD_PREFIX.length))) return { ok: true, value: text };
  return reject('bad-person', `${field} cannot be ${text}: a lead of another board is only for to-dos.`);
};

const checkPerson = (field: string, spec: ItemField, raw: string, cx: ContentContext): Verdict => {
  const text = raw.trim();
  if (text === 'owner') return { ok: true, value: text };
  if (text.startsWith(LEAD_PREFIX)) return checkLead(field, spec, text);
  if (!PERSON_PATTERN.test(text)) return reject('bad-person', `${field} is the owner or a thread id like lead, not "${text}".`);
  const thread = canonicalThread(text);
  if (cx.threads && !cx.threads.includes(thread)) return reject('unknown-person', `${thread} is not a thread of this board.`);
  return { ok: true, value: thread };
};

export { checkPerson };
