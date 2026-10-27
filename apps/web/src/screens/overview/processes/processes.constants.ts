/* @layer renderer-app @kind constants */
import { formatCpu, formatMemory, formatMinutes } from '../../../format';
import type { ProcFilters, ProcRow } from './model';

const ERROR_CLIP = 120;
const COMMAND_CLIP = 52;

const NOT_SAMPLED = 'Argus has not sampled the processes yet.';
const NONE_LISTED = 'No process matches a rule right now.';
const NONE_SHOWN = 'No process matches these filters.';
const GONE = 'That process is gone already, nothing was killed.';

const ALL_RULES = '*';

const NO_FILTERS: ProcFilters = { status: 'all', rule: '' };

const OVER_LOOK = { variant: 'urgency', color: 'warning' } as const;
const CALM_LOOK = { variant: 'normal', color: 'neutral' } as const;

const MEASURE_CELL: Readonly<Record<string, (row: ProcRow) => string>> = {
  memMB: (row) => formatMemory(row.memMB),
  cpuPct: (row) => formatCpu(row.cpuPct),
  ageMin: (row) => formatMinutes(row.ageMin),
  idleMin: (row) => formatMinutes(row.idleMin),
};

const RULE_HEADINGS = ['Rule', 'Match', 'Max memory', 'Group cap', 'Max age', 'Idle', 'Orphan', 'Hung', 'Kill', 'Note'] as const;

export {
  ALL_RULES, CALM_LOOK, COMMAND_CLIP, ERROR_CLIP, GONE, NO_FILTERS, NONE_LISTED, NONE_SHOWN, MEASURE_CELL, NOT_SAMPLED, OVER_LOOK, RULE_HEADINGS,
};
