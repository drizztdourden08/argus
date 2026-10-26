/* @layer renderer-app @kind logic */
import { CONTEXT_SHOWN_PARAGRAPHS } from './feed.constants';
import type { ContextParts } from './feed.type';

const contextParts = (context: string): ContextParts => {
  const text = context.trim();
  const paragraphs = text.includes('```') ? [text] : text.split(/\n\s*\n/).filter(Boolean);
  return {
    shown: paragraphs.slice(0, CONTEXT_SHOWN_PARAGRAPHS).join('\n\n'),
    rest: paragraphs.slice(CONTEXT_SHOWN_PARAGRAPHS).join('\n\n'),
  };
};

export { contextParts };
