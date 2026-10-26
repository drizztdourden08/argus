/* @layer renderer-app @kind logic */
import { CAPTION_MAX_CHARS } from './model.constants';

const proofCaption = (given: string, fallback: string): string => {
  const folded = (given || fallback).replace(/\s+/g, ' ').trim();
  return (folded || 'screenshot').slice(0, CAPTION_MAX_CHARS);
};

export { proofCaption };
