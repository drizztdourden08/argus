/* @layer tooling-scripts @kind logic */
import { changedFiles } from './changed-files.mjs';
import { checkTags } from './check-tags.mjs';
import { formatCommand } from './format-command.mjs';
import { listOccurrences } from './occurrences.mjs';
import { parseArgs } from './parse-args.mjs';
import { planCommands } from './plan-commands.mjs';
import { relativePath } from './relative-path.mjs';
import { runCommands } from './run-commands.mjs';
import { selectOccurrences } from './select-occurrences.mjs';
import { tagProblems } from './tag-problems.mjs';
import { usage } from './usage.mjs';
import { loadWorkspace } from './workspace.mjs';

const tagLint = (root) => {
  const { problems, withTests } = checkTags(root);
  for (const problem of problems) console.log(problem);
  console.log(`${withTests} Rust files with tests, ${problems.length} without exactly one valid tag`);
  return problems.length === 0 ? 0 : 1;
};

const filesFor = (root, options) => {
  const files = options.changed ? changedFiles(root, options.base) : options.files;
  return files.map((file) => relativePath(root, file));
};

const refuse = (problems) => {
  console.error('refusing to run files without exactly one test-tag line:');
  for (const problem of problems) console.error(`  ${problem}`);
  return 1;
};

const runTags = async (root, options) => {
  const crates = loadWorkspace(root);
  const occurrences = listOccurrences(root, crates);
  const files = filesFor(root, options);
  const selected = selectOccurrences({ crates, occurrences, depth: options.depth }, files);
  const problems = tagProblems(selected);
  if (problems.length > 0) return refuse(problems);
  const commands = planCommands(selected, occurrences, options.tag);
  if (commands.length === 0) {
    console.log('no Rust tests cover the changed files');
    return 0;
  }
  console.log(`rust-tests: ${files.length} files, tag ${options.tag}, ${commands.length} cargo commands`);
  if (options.dryRun) {
    commands.forEach((command) => console.log(formatCommand(command)));
    return 0;
  }
  return (await runCommands(root, commands)) ? 0 : 1;
};

const cli = async (root, argv) => {
  const options = parseArgs(argv);
  if (options.problem) {
    console.error(`${options.problem}\n${usage}`);
    return 2;
  }
  return options.checkTags ? tagLint(root) : runTags(root, options);
};

export { cli };
