/* @layer renderer-app @kind logic */
const queryString = (params: Record<string, string | number | undefined>): string => {
  const pairs = Object.entries(params)
    .filter((pair): pair is [string, string | number] => pair[1] !== undefined && pair[1] !== '')
    .map(([name, value]) => `${encodeURIComponent(name)}=${encodeURIComponent(String(value))}`);
  return pairs.length > 0 ? `?${pairs.join('&')}` : '';
};

export { queryString };
