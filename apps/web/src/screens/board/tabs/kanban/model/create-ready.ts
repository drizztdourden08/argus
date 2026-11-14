/* @layer renderer-app @kind logic */
const createReady = (thread: string, needsParent: boolean, parent: string): boolean => thread !== '' && (!needsParent || parent !== '');

export { createReady };
