import { ErrorMessage, err, NixError, instanceToClass } from ".";
import { NixTypeClass, NixTypeInstance } from "../lib";

export class NixOtherError {
  constructor(
    public readonly message: ErrorMessage,
    public readonly codename: string,
  ) {}

  toDefaultErrorMessage(): ErrorMessage {
    return this.message;
  }
}

export function otherError(message: string | ErrorMessage, codename: string) {
  if (typeof message === "string") {
    message = err`${message}`;
  }

  let other = new NixOtherError(message, codename);
  return new NixError(other, other.toDefaultErrorMessage());
}
