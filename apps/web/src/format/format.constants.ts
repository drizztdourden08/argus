/* @layer renderer-app @kind constants */
const SECONDS_PER_MINUTE = 60;
const MINUTES_PER_HOUR = 60;
const HOURS_PER_DAY = 24;
const TWO_DAYS_IN_HOURS = 48;
const MB_PER_GB = 1024;
const GB_WHOLE_FROM_MB = 10240;
const CPU_DECIMALS_BELOW = 10;
const ABSENT = '-';
const TIMES = '\u00d7';
const BETWEEN_DIGITS = /(?<=\d)x(?=\d)/g;
const HEX_PREFIX = /(?<![\w.])0$/;

export { ABSENT, BETWEEN_DIGITS, CPU_DECIMALS_BELOW, HEX_PREFIX, GB_WHOLE_FROM_MB, HOURS_PER_DAY, MB_PER_GB, MINUTES_PER_HOUR, SECONDS_PER_MINUTE, TIMES, TWO_DAYS_IN_HOURS };
