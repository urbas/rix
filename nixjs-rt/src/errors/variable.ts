import { ErrorMessage, err, NixError, instanceToClass, highlighted } from ".";
import { NixTypeClass, NixTypeInstance } from "../lib";

export class NixCouldntFindVariableError {
  constructor(public readonly varName: string) {}

  toDefaultErrorMessage(): ErrorMessage {
    return err`Couldn't find variable '${highlighted(this.varName)}'`;
  }
}

export function couldntFindVariableError(varName: string) {
  let error = new NixCouldntFindVariableError(varName);
  return new NixError(error, error.toDefaultErrorMessage());
}
