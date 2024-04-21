import {
  ErrorMessage,
  err,
  NixError,
  instanceToClass,
  errType,
  errTypes,
} from ".";
import { NixTypeClass, NixTypeInstance } from "../lib";

export class NixTypeMismatchError {
  constructor(
    public readonly expected: NixTypeClass[],
    public readonly got: NixTypeClass,
  ) {}

  toDefaultErrorMessage(): ErrorMessage {
    return err`Expected ${errTypes(...this.expected)}, but got ${errType(this.got)}`;
  }
}

export function typeMismatchError(
  got: NixTypeClass | NixTypeInstance,
  expected: NixTypeClass | NixTypeClass[],
  message?: ErrorMessage,
) {
  if (!Array.isArray(expected)) {
    expected = [expected];
  }

  const error = new NixTypeMismatchError(expected, instanceToClass(got));
  return new NixError(error, message ?? error.toDefaultErrorMessage());
}

/** Similar to a type mismatch error, but with expected being [] and the message is required */
export function invalidTypeError(
  got: NixTypeClass | NixTypeInstance,
  message: ErrorMessage,
) {
  const error = new NixTypeMismatchError([], instanceToClass(got));
  return new NixError(error, message);
}
