/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { boardCode } from '../../src/shell/records/behavior/board-code';
import { hasRecords } from '../../src/shell/records/behavior/has-records';
import { parseRecord } from '../../src/shell/records/behavior/parse-record';
import { stickyCode } from '../../src/shell/records/behavior/sticky-code';
import type { RecordRef } from '../../src/shell/records/records.type';

const must = (token: string): RecordRef => {
  const found = parseRecord(token);
  if (!found) throw new Error(`not a record: ${token}`);
  return found;
};

describe('I-85 the code a pane opens on the overview page', { tags: ['critical'] }, () => {
  it('keeps the board in front of the code so the overview knows which board to read', () => {
    expect(boardCode(must('rotp/I-12'))).toBe('rotp/I-12');
    expect(boardCode(must('I-12'))).toBe('I-12');
  });

  it('keeps the board of the open record when the pane opens a code without one', () => {
    expect(stickyCode('Q-4', null, must('rotp/I-12'))).toBe('rotp/Q-4');
  });

  it('leaves a code that names its own board, a board page and a closed pane alone', () => {
    expect(stickyCode('brock/Q-4', null, must('rotp/I-12'))).toBe('brock/Q-4');
    expect(stickyCode('Q-4', 'rotp', must('rotp/I-12'))).toBe('Q-4');
    expect(stickyCode('Q-4', null, null)).toBe('Q-4');
  });

  it('leaves text that is not a record alone', () => {
    expect(stickyCode('hello', null, must('rotp/I-12'))).toBe('hello');
  });
});

describe('I-85 which text carries a reference', { tags: ['critical'] }, () => {
  it('sees an id inside a sentence and not a plain word', () => {
    expect(hasRecords('Blocked by I-3 today')).toBe(true);
    expect(hasRecords('rotp/I-9 is late')).toBe(true);
    expect(hasRecords('Ship it today, not UTF-8')).toBe(false);
  });
});
