import { ErrorMessage, err, NixError, instanceToClass } from ".";
import { NixTypeClass, NixTypeInstance } from "../lib";

export class NixOtherError {
  constructor(public readonly message: string) {}

  toDefaultErrorMessage(): ErrorMessage {
    return err`${this.message}`;
  }
}

export function otherError(message: string) {
  let other = new NixOtherError(message);
  return new NixError(other, other.toDefaultErrorMessage());
}
