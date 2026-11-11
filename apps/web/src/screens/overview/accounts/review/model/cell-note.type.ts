/* @layer renderer-app @kind types */
type NoteTest = 'target-missing' | 'target-present' | 'other-present' | 'active' | 'archived' | 'not-linked' | 'linked';

interface NoteRule {
  test: NoteTest;
  note: string;
}

export type { NoteRule, NoteTest };
