/* @layer tooling-scripts @kind logic */
const quote = (arg) => (/^[\w.:=/@-]+$/.test(arg) ? arg : `"${arg}"`);

const formatCommand = (command) => `cargo ${command.args.map(quote).join(' ')}`;

export { formatCommand };
