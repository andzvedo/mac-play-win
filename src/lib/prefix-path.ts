export function prefixPathFor(prefixesDir: string, gameId: string): string {
  const base = prefixesDir.replace(/[/\\]+$/, "");
  return `${base}/${gameId}`;
}
