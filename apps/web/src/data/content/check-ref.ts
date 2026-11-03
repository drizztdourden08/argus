/* @layer renderer-app @kind logic */
import type { ItemField } from '../registry';
import { REF_PATTERNS } from './content.constants';
import type { ContentContext, Verdict } from './content.type';
import { reject } from './reject';

const checkRef = (field: string, spec: ItemField, raw: string, cx: ContentContext): Verdict => {
  const text = raw.trim();
  const kinds = spec.to ?? [];
  if (!kinds.some((kind) => REF_PATTERNS.get(kind)?.test(text))) return reject('bad-ref', `${field} is an id of ${kinds.join(' or ')}, not "${text}".`);
  if (cx.ids && !cx.ids.includes(text)) return reject('unknown-ref', `${text} is not on this board.`);
  return { ok: true, value: text };
};

export { checkRef };
