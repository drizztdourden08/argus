/* @layer renderer-app @kind test */
const ownerItem = (id: string, extra: object = {}) => ({ id, title: `Title of ${id}`, status: 'doing', thread: 'lead', type: 'task', ...extra });

export { ownerItem };
