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
import { NixFunctionCallWithoutArgumentError } from "./function";
import { NixOtherError } from "./other";
import { NixTypeMismatchError } from "./typeError";
import { NixCouldntFindVariableError } from "./variable";

/** A helper to convert instances to their respective class in a type-safe way, for better error messages */
export function instanceToClass(instance: NixTypeInstance | NixTypeClass) {
  if (instance instanceof NixBool) {
    return NixBool;
  } else if (instance instanceof NixFloat) {
    return NixFloat;
  } else if (instance instanceof NixInt) {
    return NixInt;
  } else if (instance instanceof NixList) {
    return NixList;
  } else if (instance instanceof NixNull) {
    return NixNull;
  } else if (instance instanceof NixString) {
    return NixString;
  } else if (instance instanceof Path) {
    return Path;
  } else if (instance instanceof Lambda) {
    return Lambda;
  } else if (instance instanceof Attrset) {
    return Attrset;
  } else if (instance instanceof Lazy) {
    return instanceToClass(instance.toStrict());
  } else {
    return instance;
  }
}

type PlainErrorMessagePart = {
  kind: "plain";
  value: string;
};

type HighlightedErrorMessagePart = {
  kind: "highlighted";
  value: string;
};

type ErrorMessagePart = PlainErrorMessagePart | HighlightedErrorMessagePart;
export type ErrorMessage = ErrorMessagePart[];

/** A hack-y way of finding whether an object is an ErrorMessagePart. Essential for error message building. */
function isErrorMessagePart(part: any): part is ErrorMessagePart {
  return (
    typeof part === "object" &&
    part !== null &&
    "kind" in part &&
    typeof part.kind === "string"
  );
}

function isErrorMessage(parts: any): parts is ErrorMessage {
  if (!Array.isArray(parts)) {
    return false;
  }

  return parts.every(isErrorMessagePart);
}

type ErrorMessageBuilderPart =
  | string
  | NixTypeClass
  | NixTypeClass[]
  | NixTypeInstance
  | ErrorMessage;

/**
 * Generates a human-readable representation of multiple types
 *
 * E.g. `[NixInt, NixFloat, NixString]` becomes `a number, a float, or a string`
 */
export function classListToErrorMessage(
  classes: NixTypeClass | NixTypeClass[],
): ErrorMessage {
  if (!Array.isArray(classes)) {
    return err`${classes}`;
  }

  if (classes.length === 0) {
    throw new Error("classListToErrorMessage: classes array is empty");
  }

  if (classes.length === 1) {
    return err`${classes[0]}`;
  }

  // Dynamically build the error message, separating with commas and "or"
  const message: ErrorMessage = [];
  for (let i = 0; i < classes.length; i++) {
    if (i === classes.length - 1) {
      message.push(...err`or`);
    } else if (i > 0) {
      message.push(...err`,`);
    }
    message.push(...err`${classes[i]}`);
  }
}

/** Takes builder parts (strings, classes, instances, other error messages) and creates a new error message array */
function errorBuilderPartToMessagePart(
  part: ErrorMessageBuilderPart,
): ErrorMessagePart[] {
  if (typeof part === "string") {
    return [{ kind: "plain", value: part }];
  } else if (isErrorMessage(part)) {
    // This is a child message, being inserted in
    return part;
  } else if (Array.isArray(part)) {
    // This is a list of classes
    classListToErrorMessage(part).map<HighlightedErrorMessagePart>((part) => ({
      kind: "highlighted",
      value: part.value,
    }));
  } else {
    let Class = instanceToClass(part);
    return [{ kind: "highlighted", value: Class.toHumanReadable() }];
  }
}

/**
 * Tag function for building error messages, especially type mismatch messages.
 *
 * # Example:
 * ```ts
 * err`Expected ${NixInt}, but got ${value}`
 * ```
 */
export function err(
  strings: readonly string[],
  ...parts: readonly ErrorMessageBuilderPart[]
): ErrorMessage {
  // Join the strings and parts together
  const messageParts: ErrorMessagePart[] = [];
  for (let i = 0; i < strings.length; i++) {
    messageParts.push(...errorBuilderPartToMessagePart(strings[i]));
    if (i < parts.length) {
      messageParts.push(...errorBuilderPartToMessagePart(parts[i]));
    }
  }

  return messageParts;
}

export function highlighted(message: string): ErrorMessage {
  return [{ kind: "highlighted", value: message }];
}

export function stringifyErrorMessage(message: ErrorMessage): string {
  return message.map((part) => part.value).join("");
}

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
    message: ErrorMessage,
  ) {
    // TODO: In the future, make error messages have color highlighting for special error parts
    const messageString = message.map((part) => part.value).join("");

    super(messageString);
  }
}
