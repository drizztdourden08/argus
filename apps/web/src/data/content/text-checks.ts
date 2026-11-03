/* @layer renderer-app @kind logic */
import type { ContentKind, ItemField } from '../registry';
import { checkChoice } from './check-choice';
import { checkDate } from './check-date';
import { checkLine } from './check-line';
import { checkPerson } from './check-person';
import { checkRef } from './check-ref';
import { checkSlug } from './check-slug';
import type { ContentContext, Verdict } from './content.type';
import { checkRich } from './rich';

type TextCheck = (field: string, spec: ItemField, text: string, cx: ContentContext) => Verdict;

const textChecks: Record<Exclude<ContentKind, 'records'>, TextCheck> = {
  line: checkLine,
  slug: checkSlug,
  rich: checkRich,
  enum: checkChoice,
  date: checkDate,
  ref: checkRef,
  refs: checkRef,
  person: checkPerson,
};

export { textChecks };
