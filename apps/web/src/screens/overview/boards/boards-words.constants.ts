/* @layer renderer-app @kind constants */
const WORDS = {
  updated: (ago: string): string => `Updated ${ago}`,
  leadSeen: (ago: string): string => `Lead last seen ${ago}`,
  synced: (ago: string): string => `Last sync ${ago}`,
  syncDue: (ago: string): string => `Last sync ${ago}, due now`,
  neverSynced: 'Never synced',
  progressDone: (done: number, total: number): string => `${done} of ${total} done`,
  progressReview: (count: number): string => `${count} in review`,
  progressDoing: (count: number): string => `${count} in progress`,
} as const;

export { WORDS };
