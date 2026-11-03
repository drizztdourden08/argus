/* @layer renderer-app @kind logic */
const fieldLabel = (name: string): string => {
  const words = name.split(/(?=[A-Z])/).join(' ').toLowerCase();
  return `${words.charAt(0).toUpperCase()}${words.slice(1)}`;
};

export { fieldLabel };
