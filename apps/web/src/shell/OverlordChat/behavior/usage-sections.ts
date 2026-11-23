/* @layer renderer-app @kind logic */
import type { OverlordUsage, UsageTotals } from '../../../data/overlord';
import { USAGE_TITLES, USAGE_WORDS } from '../OverlordChat.constants';
import type { UsageRow, UsageSection } from '../chat-settings.type';
import { formatClock } from './format-clock';
import { formatTokens } from './format-tokens';
import { formatUsd } from './format-usd';

const sessionRows = (totals: UsageTotals): UsageRow[] => [
  { label: USAGE_WORDS.input, value: formatTokens(totals.inputTokens) },
  { label: USAGE_WORDS.output, value: formatTokens(totals.outputTokens) },
  { label: USAGE_WORDS.cacheRead, value: formatTokens(totals.cacheReadTokens) },
  { label: USAGE_WORDS.cacheWrite, value: formatTokens(totals.cacheWriteTokens) },
  { label: USAGE_WORDS.cost, value: formatUsd(totals.costUsd) },
  { label: USAGE_WORDS.turns, value: String(totals.turns) },
  { label: USAGE_WORDS.duration, value: formatClock(totals.durationMs) },
];

const lastTurnRows = (totals: UsageTotals): UsageRow[] => [
  { label: USAGE_WORDS.input, value: formatTokens(totals.inputTokens) },
  { label: USAGE_WORDS.output, value: formatTokens(totals.outputTokens) },
  { label: USAGE_WORDS.cost, value: formatUsd(totals.costUsd) },
];

const monthRows = (month: OverlordUsage['month']): UsageRow[] => [
  { label: USAGE_WORDS.spent, value: formatUsd(month.spentUsd) },
  { label: USAGE_WORDS.budget, value: month.budgetUsd === null ? USAGE_WORDS.noBudget : formatUsd(month.budgetUsd) },
];

const contextRows = (usage: OverlordUsage): UsageRow[] => [
  { label: USAGE_WORDS.context, value: `${formatTokens(usage.context.usedTokens)} of ${formatTokens(usage.context.windowTokens)} tokens` },
  { label: USAGE_WORDS.model, value: usage.model },
];

const usageSections = (usage: OverlordUsage): UsageSection[] => {
  const sections: UsageSection[] = [
    { title: USAGE_TITLES.context, rows: contextRows(usage) },
    { title: USAGE_TITLES.session, rows: sessionRows(usage.session) },
  ];
  if (usage.lastTurn) sections.push({ title: USAGE_TITLES.lastTurn, rows: lastTurnRows(usage.lastTurn) });
  sections.push({ title: USAGE_TITLES.month, rows: monthRows(usage.month) });
  return sections;
};

export { usageSections };
