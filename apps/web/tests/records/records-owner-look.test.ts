/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { assigneeLook } from '../../src/shell/records/behavior/assignee-look';
import { peopleOptions } from '../../src/shell/records/pane/model/people-options';
import { ownerMark } from '../../src/screens/board/tabs/kanban/signals/owner-mark';

const alex = { name: 'Alex', initials: 'AX', avatar: null };

describe('I-111 the owner shows by name', { tags: ['critical'] }, () => {
  it('names the owner mark by the profile and keeps the sentence', () => {
    expect(assigneeLook('owner', 'argus', [])).toMatchObject({ label: 'You', owner: true });
    expect(assigneeLook('owner', 'argus', [], alex)).toMatchObject({ label: 'Alex', tip: 'Assigned to you', owner: true });
  });

  it('labels the picker entry with the name and you, or just You by default', () => {
    expect(peopleOptions([])[0]).toEqual({ value: 'owner', label: 'You' });
    expect(peopleOptions([], undefined, alex)[0]).toEqual({ value: 'owner', label: 'Alex (you)' });
  });

  it('draws the signal chip owner from the profile face', () => {
    expect(ownerMark(alex)).toMatchObject({ avatar: { icon: 'user', size: 'xs', variant: 'owner', label: 'Alex' } });
    expect(ownerMark({ ...alex, avatar: 'initials' })).toMatchObject({ avatar: { initials: 'AX' } });
  });
});
