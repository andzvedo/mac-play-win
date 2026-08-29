export function assertNever(value: never): never {
  throw new Error(`Valor não tratado: ${String(value)}`);
}
