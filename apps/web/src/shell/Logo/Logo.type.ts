/* @layer renderer-app @kind types */
type LogoSize = 'sm' | 'md' | 'lg';

type LogoTone = 'text' | 'primary';

interface LogoProps {
  size?: LogoSize;
  tone?: LogoTone;
  label?: string;
}

interface LogoArt {
  d: string[];
  circles: { cx: number; cy: number; r: number }[];
}

export type { LogoArt, LogoProps };
