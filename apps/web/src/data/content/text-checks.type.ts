/* @layer renderer-app @kind types */
import type { ItemField } from '../registry';
import type { ContentContext, Verdict } from './content.type';

type TextCheck = (field: string, spec: ItemField, text: string, cx: ContentContext) => Verdict;

export type { TextCheck };
