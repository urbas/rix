import { ErrorMessage, err, NixError, instanceToClass, highlighted } from ".";
import { NixTypeClass, NixTypeInstance } from "../lib";

export class NixAttributeAlreadyDefinedError {
  constructor(public readonly attrPath: string[]) {}

  toDefaultErrorMessage(): ErrorMessage {
    return err`Attribute '${highlighted(this.attrPath.join("."))}' is already defined'`;
  }
}

export function attributeAlreadyDefinedError(attrPath: string[]) {
  let error = new NixAttributeAlreadyDefinedError(attrPath);
  return new NixError(error, error.toDefaultErrorMessage());
}

export class NixMissingAttributeError {
  constructor(public readonly attrPath: string[]) {}

  toDefaultErrorMessage(): ErrorMessage {
    return err`Attribute '${highlighted(this.attrPath.join("."))}' is missing`;
  }
}

export function missingAttributeError(attrPath: string[]) {
  let error = new NixMissingAttributeError(attrPath);
  return new NixError(error, error.toDefaultErrorMessage());
}
