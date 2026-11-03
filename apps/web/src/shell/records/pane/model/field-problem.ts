/* @layer renderer-app @kind logic */
import type { ContentContext } from '../../../../data/content';
import { fieldVerdict } from './field-verdict';

const fieldProblem = (name: string, text: string, cx: ContentContext): string | undefined => {
  const verdict = fieldVerdict(name, text, cx);
  return verdict.ok ? undefined : verdict.message;
};

export { fieldProblem };
