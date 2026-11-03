/* @layer renderer-app @kind logic */
const ownEntry = <T>(table: Readonly<Record<string, T>>, key: string): T | undefined => (Object.hasOwn(table, key) ? table[key] : undefined);

export { ownEntry };
