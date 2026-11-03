/* @layer renderer-app @kind logic */
import { MONTH_DAYS } from './content.constants';

const isLeap = (year: number): boolean => year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);

const onCalendar = (text: string): boolean => {
  const [year = Number.NaN, month = Number.NaN, day = Number.NaN] = text.split('-').map(Number);
  const days = MONTH_DAYS[month - 1];
  if (days === undefined) return false;
  const last = month === 2 && isLeap(year) ? 29 : days;
  return day >= 1 && day <= last;
};

export { onCalendar };
