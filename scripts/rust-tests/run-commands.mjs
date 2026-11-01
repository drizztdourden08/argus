/* @layer tooling-scripts @kind logic */
import { formatCommand } from './format-command.mjs';
import { runCommand } from './run-command.mjs';

const summaryLine = (crate, entry) => {
  const { passed, failed, ignored } = entry.counts;
  const state = entry.failed ? 'FAILED' : 'ok';
  const codes = entry.codes.filter((code) => code !== 0).join(', ');
  const exit = codes ? `, exit ${codes}` : '';
  return `${crate}: ${state}${exit}, ${passed} passed, ${failed} failed, ${ignored} ignored`;
};

const addTo = (summary, command, result) => {
  const entry = summary.get(command.crate) ?? { failed: false, codes: [], counts: { passed: 0, failed: 0, ignored: 0 } };
  entry.failed = entry.failed || result.code !== 0;
  entry.codes.push(result.code);
  for (const key of Object.keys(entry.counts)) entry.counts[key] += result.counts[key];
  summary.set(command.crate, entry);
};

const runCommands = async (root, commands) => {
  const summary = new Map();
  for (const command of commands) {
    console.log(`\n> ${formatCommand(command)}`);
    addTo(summary, command, await runCommand(root, command));
  }
  console.log('\nsummary');
  for (const [crate, entry] of summary) console.log(summaryLine(crate, entry));
  return [...summary.values()].every((entry) => !entry.failed);
};

export { runCommands };
