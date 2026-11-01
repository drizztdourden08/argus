/* @layer tooling-scripts @kind logic */
const runTags = ['critical', 'standard', 'e2e'];

const takeValues = (argv, from) => {
  const values = [];
  for (let index = from; index < argv.length && !argv[index].startsWith('--'); index += 1) values.push(argv[index]);
  return values;
};

const flag = (name, value) => (options, _argv, index) => {
  options[name] = value;
  return index + 1;
};

const withValue = (name, convert) => (options, argv, index) => {
  options[name] = convert(argv[index + 1]);
  return index + 2;
};

const handlers = {
  '--changed': flag('changed', true),
  '--dry-run': flag('dryRun', true),
  '--check-tags': flag('checkTags', true),
  '--tag': withValue('tag', String),
  '--base': withValue('base', String),
  '--depth': withValue('depth', Number),
  '--files': (options, argv, index) => {
    const values = takeValues(argv, index + 1);
    options.files = values;
    return index + 1 + values.length;
  },
};

const problemOf = (options) => {
  if (options.checkTags) return null;
  if (!runTags.includes(options.tag)) return 'pass --tag critical, standard or e2e';
  if (!Number.isInteger(options.depth) || options.depth < 0) return '--depth takes a whole number of zero or more';
  if (options.changed === (options.files !== null)) return 'pass exactly one of --changed or --files';
  return null;
};

const parseArgs = (argv) => {
  const options = { tag: null, changed: false, files: null, base: null, depth: 1, dryRun: false, checkTags: false };
  let index = 0;
  while (index < argv.length) {
    const handler = handlers[argv[index]];
    if (!handler) return { ...options, problem: `unknown argument ${argv[index]}` };
    index = handler(options, argv, index);
  }
  return { ...options, problem: problemOf(options) };
};

export { parseArgs };
