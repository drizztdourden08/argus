/* @layer renderer-app @kind logic */
const isKey = (value: unknown): value is string => typeof value === 'string' && value !== '';

const fieldKeys = (doc: Readonly<Record<string, unknown>> | null, field: string | undefined): string[] => {
  const value = doc && field ? doc[field] : undefined;
  if (Array.isArray(value)) return value.filter(isKey);
  return isKey(value) ? [value] : [];
};

export { fieldKeys };
