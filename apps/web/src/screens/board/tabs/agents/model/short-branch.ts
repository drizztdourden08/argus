/* @layer renderer-app @kind logic */
const shortBranch = (branch: string): string => {
  const parts = branch.split('/');
  return parts.length > 1 ? parts.slice(1).join('/') : branch;
};

export { shortBranch };
