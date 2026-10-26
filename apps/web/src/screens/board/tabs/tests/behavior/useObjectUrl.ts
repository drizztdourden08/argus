/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';

const useObjectUrl = (file: File | null): string | null => {
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    if (!file || typeof URL.createObjectURL !== 'function') {
      setUrl(null);
      return undefined;
    }
    const made = URL.createObjectURL(file);
    setUrl(made);
    return () => URL.revokeObjectURL(made);
  }, [file]);
  return url;
};

export { useObjectUrl };
