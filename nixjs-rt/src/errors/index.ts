import {
  Attrset,
  Lambda,
  Lazy,
  NixBool,
  NixFloat,
  NixInt,
  NixList,
  NixNull,
  NixString,
  NixType,
  NixTypeClass,
  NixTypeInstance,
  Path,
} from "../lib";
import { NixAbortError } from "./abort";
import {
  NixAttributeAlreadyDefinedError,
  NixMissingAttributeError,
} from "./attribute";
import { ErrorMessage } from "./errorMessage";
import { NixFunctionCallWithoutArgumentError } from "./function";
import { NixOtherError } from "./other";
import { NixTypeMismatchError } from "./typeError";
import { NixCouldntFindVariableError } from "./variable";

export * from "./errorMessage";

type NixErrorKind =
  | NixTypeMismatchError
  | NixAbortError
  | NixOtherError
  | NixMissingAttributeError
  | NixAttributeAlreadyDefinedError
  | NixFunctionCallWithoutArgumentError
  | NixCouldntFindVariableError;

/** The base error class. This class gets parsed in rix by Rust code. */
export class NixError extends Error {
  constructor(
    public readonly kind: NixErrorKind,
    public readonly richMessage: ErrorMessage,
  ) {
    // TODO: In the future, make error messages have color highlighting for special error parts
    const messageString = richMessage.map((part) => part.value).join("");

    super(messageString);
  }
}
