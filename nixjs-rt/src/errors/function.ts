import { ErrorMessage, err, NixError, instanceToClass, highlighted } from ".";
import { NixTypeClass, NixTypeInstance } from "../lib";

export class NixFunctionCallWithoutArgumentError {
  constructor(public readonly argument: string) {}

  toDefaultErrorMessage(): ErrorMessage {
    return err`Function call is missing required argument '${highlighted(this.argument)}'`;
  }
}

export function functionCallWithoutArgumentError(argument: string) {
  let error = new NixFunctionCallWithoutArgumentError(argument);
  return new NixError(error, error.toDefaultErrorMessage());
}
