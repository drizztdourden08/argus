/* @layer renderer-app @kind logic */
import type { ContentKind } from '../registry';
import { checkChoice } from './check-choice';
import { checkDate } from './check-date';
import { checkLine } from './check-line';
import { checkPerson } from './check-person';
import { checkRef } from './check-ref';
import { checkSlug } from './check-slug';
import { checkRich } from './rich';
import type { TextCheck } from './text-checks.type';

const textChecks: Record<Exclude<ContentKind, 'flag' | 'records'>, TextCheck> = {
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
