export function isWindowsExePath(path: string): boolean {
  const trimmed = path.trim();
  if (!trimmed) {
    return false;
  }
  const base = trimmed.split(/[/\\]/).pop() ?? trimmed;
  return /\.exe$/i.test(base);
}
