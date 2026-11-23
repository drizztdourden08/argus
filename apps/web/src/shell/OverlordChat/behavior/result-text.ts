/* @layer renderer-app @kind logic */
import { clipText } from '../../../format';
import { SUMMARY_MAX } from '../OverlordChat.constants';

const partText = (part: unknown): string => {
  const text = part && typeof part === 'object' ? (part as { text?: unknown }).text : undefined;
  return typeof text === 'string' ? text : '';
};

const joined = (content: unknown): string => {
  if (typeof content === 'string') return content;
  return Array.isArray(content) ? content.map(partText).join('\n') : '';
};

const resultText = (content: unknown): string => clipText(joined(content).trim().split('\n')[0] ?? '', SUMMARY_MAX);

export { resultText };
