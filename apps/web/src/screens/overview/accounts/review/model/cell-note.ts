/* @layer renderer-app @kind logic */
import type { ReviewCell } from '../../../../../data/types';
import { NOTE_RULES } from './cell-note.constants';
import type { NoteTest } from './cell-note.type';

const byTarget = (test: NoteTest, here: boolean, present: boolean): boolean | null => {
  if (test === 'target-missing') return here && !present;
  if (test === 'target-present') return here && present;
  if (test === 'other-present') return !here && present;
  return null;
};

const byState = (test: NoteTest, cell: ReviewCell): boolean => {
  if (test === 'active') return cell.present === true && cell.archived !== true;
  if (test === 'archived') return cell.present === true && cell.archived === true;
  if (test === 'not-linked') return cell.status === 'not_subthread';
  return cell.status === 'subthread';
};

const cellNote = (choice: string, key: string, cell: ReviewCell): string | null => {
  const [verb = '', target] = choice.split(':');
  const rule = NOTE_RULES[verb];
  if (!rule) return null;
  const hit = byTarget(rule.test, target === key, cell.present === true) ?? byState(rule.test, cell);
  return hit ? rule.note : null;
};

export { cellNote };
