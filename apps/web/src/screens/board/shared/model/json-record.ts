/* @layer renderer-app @kind logic */
const jsonRecord = (raw: string | null): Record<string, unknown> => {
  try {
    const parsed: unknown = JSON.parse(raw ?? '{}');
    return parsed && typeof parsed === 'object' && !Array.isArray(parsed) ? (parsed as Record<string, unknown>) : {};
  } catch {
    return {};
  }
};

export { jsonRecord };
