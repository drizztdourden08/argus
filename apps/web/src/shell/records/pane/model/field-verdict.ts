/* @layer renderer-app @kind logic */
import { checkField } from '../../../../data/content';
import type { ContentContext, Verdict } from '../../../../data/content';
import { textValue } from './text-value';

const fieldVerdict = (name: string, text: string, cx: ContentContext): Verdict => checkField(name, textValue(name, text), cx);

export { fieldVerdict };
