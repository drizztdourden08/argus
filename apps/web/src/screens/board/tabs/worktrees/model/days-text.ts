/* @layer renderer-app @kind logic */
const daysText = (days: number): string => `${days} ${days === 1 ? 'day' : 'days'}`;

export { daysText };
