import { ErrorMessage, err, NixError, instanceToClass, highlighted } from ".";
import { NixTypeClass, NixTypeInstance } from "../lib";

export class NixCouldntFindVariableError {
  constructor(public readonly argument: string) {}

  toDefaultErrorMessage(): ErrorMessage {
    return err`Couldn't find variable '${highlighted(this.argument)}'`;
  }
}

export function couldntFindVariableError(argument: string) {
  let error = new NixCouldntFindVariableError(argument);
  return new NixError(error, error.toDefaultErrorMessage());
}
