/* @layer renderer-app @kind constants */
import type { BadgeColor, IconName, ListItemHeader, ListItemShape, SegmentOption, StatusTone, TagLook } from '@drizztdourden08/tessera';
import type { RunTone } from '../../../data/runs';
import { BAD_LOOK, CALM_LOOK } from '../../../shell';
import type { ChipTone, LeadState, SendKind } from './model';

const SENT_LIMIT = 40;

const WARN_LOOK = { variant: 'urgency', color: 'warning' } as const;
const GOOD_LOOK = { variant: 'urgency', color: 'success' } as const;
const INFO_LOOK = { variant: 'urgency', color: 'info' } as const;

const STATE_TONE: Readonly<Record<LeadState, StatusTone>> = {
  running: 'success',
  idle: 'info',
  paused: 'warning',
  stale: 'warning',
  'not started': 'neutral',
};

const DOT_COLOR: Readonly<Record<LeadState, BadgeColor>> = {
  running: 'success',
  idle: 'info',
  paused: 'warning',
  stale: 'warning',
  'not started': 'tame',
};

const RUN_DOT: Readonly<Record<RunTone, BadgeColor>> = {
  success: 'success',
  info: 'info',
  warning: 'warning',
  danger: 'danger',
  neutral: 'tame',
};

const REGISTRY_LOOK: Readonly<Record<ChipTone, TagLook>> = {
  success: CALM_LOOK,
  neutral: CALM_LOOK,
  warning: WARN_LOOK,
  info: WARN_LOOK,
  danger: BAD_LOOK,
};

const REGISTRY_DOT: Readonly<Record<ChipTone, BadgeColor>> = {
  success: 'success',
  neutral: 'tame',
  warning: 'warning',
  info: 'warning',
  danger: 'danger',
};

const REGISTRY_ICON: Readonly<Partial<Record<ChipTone, IconName>>> = { danger: 'triangle-alert', info: 'loader-circle' };

const CARD_COLUMN_MIN = 192;
const NARROW_COLUMNS = 2;
const NARROW_QUERY = '(max-width: 479px)';

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
  CARD_COLUMN_MIN, REGISTRY_DOT, REGISTRY_ICON, REGISTRY_LOOK, DOT_COLOR, GOOD_LOOK, INFO_LOOK, KIND_WORD, NARROW_COLUMNS, NARROW_QUERY, NO_PLAY, NO_PROJECT, NOTHING_SENT, RUN_DOT, RUNNING_HEADER, SEND_KINDS,
  SEND_MESSAGE, SENT_HEADER, SENT_LIMIT, SENT_SHAPE, STATE_TONE, WARN_LOOK,
};
