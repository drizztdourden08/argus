/* @layer renderer-app @kind constants */
import type { ListItemHeader, ListItemShape, SegmentOption } from '@drizztdourden08/tessera';
import type { SendKind } from './model';

const SENT_LIMIT = 40;

const WARN_LOOK = { variant: 'urgency', color: 'warning' } as const;
const CALM_LOOK = { variant: 'normal', color: 'neutral' } as const;
const GOOD_LOOK = { variant: 'urgency', color: 'success' } as const;
const BAD_LOOK = { variant: 'urgency', color: 'danger' } as const;
const INFO_LOOK = { variant: 'urgency', color: 'info' } as const;

const STATE_LOOK = {
  running: GOOD_LOOK,
  idle: CALM_LOOK,
  paused: INFO_LOOK,
  stale: WARN_LOOK,
  'not started': CALM_LOOK,
} as const;

const NO_PROJECT = 'No project has a board yet.';
const NOTHING_SENT = 'Nothing sent down yet.';

const SEND_MESSAGE = 'It reaches the lead at once; a lead that is asleep reads it when it wakes.';

const SEND_KINDS: SegmentOption<SendKind>[] = [
  { value: 'decision', label: 'Decision' },
  { value: 'followup', label: 'Note' },
];

const NO_PLAY = 'master has no Play: its command spans several lines, or its run name is taken (coord launcher show master)';

const RUNNING_HEADER: ListItemHeader = {
  icon: 'Status',
  main: 'Run',
  columns: [{ label: 'Project' }, { label: 'Elapsed', align: 'end' }],
};

const SENT_HEADER: ListItemHeader = {
  icon: 'Kind',
  main: 'Text',
  columns: [{ label: 'Status' }, { label: 'From', align: 'end' }],
};

const SENT_SHAPE: ListItemShape = { icon: true, columns: 2, action: false };

const KIND_WORD: Readonly<Record<string, string>> = { decision: 'decision', followup: 'note' };

export {
  BAD_LOOK, CALM_LOOK, GOOD_LOOK, INFO_LOOK, KIND_WORD, NO_PLAY, NO_PROJECT, NOTHING_SENT, RUNNING_HEADER, SEND_KINDS, SEND_MESSAGE, SENT_HEADER, SENT_LIMIT, SENT_SHAPE, STATE_LOOK, WARN_LOOK,
};
