import type { PonteErrorCode } from "./types";

export class PonteError extends Error {
  readonly code: PonteErrorCode;

  constructor(code: PonteErrorCode, message: string) {
    super(message);
    this.name = "PonteError";
    this.code = code;
  }
}

export function isPonteError(error: unknown): error is PonteError {
  return error instanceof PonteError;
}

export function toPonteError(error: unknown): PonteError {
  if (error instanceof PonteError) {
    return error;
  }
  if (error instanceof Error) {
    return new PonteError("unknown", error.message);
  }
  return new PonteError("unknown", "Algo deu errado.");
}
