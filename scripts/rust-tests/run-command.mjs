/* @layer tooling-scripts @kind logic */
import { spawn } from 'node:child_process';
import { countResults } from './count-results.mjs';

const runCommand = (root, command) => new Promise((resolve) => {
  const counts = { passed: 0, failed: 0, ignored: 0 };
  const child = spawn('cargo', command.args, { cwd: root, stdio: ['ignore', 'pipe', 'pipe'] });
  let carry = '';
  child.stdout.on('data', (chunk) => {
    process.stdout.write(chunk);
    carry = countResults(carry + chunk.toString(), counts);
  });
  child.stderr.on('data', (chunk) => process.stderr.write(chunk));
  child.on('error', () => resolve({ code: 127, counts }));
  child.on('close', (code) => resolve({ code: code ?? 1, counts }));
});

export { runCommand };
