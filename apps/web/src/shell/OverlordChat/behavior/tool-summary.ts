/* @layer renderer-app @kind logic */
import { clipText } from '../../../format';
import { SUMMARY_FIELDS, SUMMARY_MAX } from '../OverlordChat.constants';

const isText = (value: unknown): value is string => typeof value === 'string' && value.trim() !== '';

const toolSummary = (input: unknown): string => {
  if (!input || typeof input !== 'object') return '';
  const fields = input as Record<string, unknown>;
  const first = SUMMARY_FIELDS.map((field) => fields[field]).find(isText) ?? Object.values(fields).find(isText);
  return isText(first) ? clipText(first.replace(/\s+/g, ' ').trim(), SUMMARY_MAX) : '';
};

export { toolSummary };
