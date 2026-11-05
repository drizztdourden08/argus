/* @layer renderer-app @kind logic */
const parseFilters = <F extends object>(raw: string, fallback: F): F => {
  try {
    const parsed: unknown = JSON.parse(raw);
    return parsed !== null && typeof parsed === 'object' ? { ...fallback, ...(parsed as Partial<F>) } : fallback;
  } catch {
    return fallback;
  }
};

export { parseFilters };
