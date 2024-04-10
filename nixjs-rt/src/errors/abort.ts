import { ErrorMessage, err, NixError, instanceToClass } from ".";
import { NixTypeClass, NixTypeInstance } from "../lib";

export class NixAbortError {
  constructor(public readonly message: string) {}

  toDefaultErrorMessage(): ErrorMessage {
    return err`Aborted: '${this.message}'`;
  }
}

export function abortError(message: string) {
  let abort = new NixAbortError(message);
  return new NixError(abort, abort.toDefaultErrorMessage());
}
