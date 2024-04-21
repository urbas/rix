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
  NixTypeClass,
  NixTypeInstance,
  Path,
} from "../lib";

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

export function stringifyErrorMessage(message: ErrorMessage): string {
  return message.map((part) => part.value).join("");
}

type ErrorMessageBuilderPart =
  | string
  // | NixTypeClass
  // | NixTypeClass[]
  // | NixTypeInstance
  | ErrorMessage;

function builderPartToErrMessage(part: ErrorMessageBuilderPart): ErrorMessage {
  if (typeof part === "string") {
    return [{ kind: "plain", value: part }];
  } else {
    return part;
  }
}

/**
 * Tag function for building error messages, especially type mismatch messages.
 *
 * # Example:
 * ```ts
 * err`Expected ${errTypes(NixInt, NixString)}, but got ${errType(value)}`
 * ```
 */
export function err(
  strings: readonly string[],
  ...parts: readonly ErrorMessageBuilderPart[]
): ErrorMessage {
  // Join the strings and parts together
  const messageParts: ErrorMessagePart[] = [];
  for (let i = 0; i < strings.length; i++) {
    messageParts.push(...builderPartToErrMessage(strings[i]));
    if (i < parts.length) {
      messageParts.push(...builderPartToErrMessage(parts[i]));
    }
  }

  return messageParts;
}

/**
 * Generates a highlighted human-readable representation of a single type
 *
 * E.g. `NixInt` becomes `a number`
 */
export function errType(type: NixTypeClass | NixTypeInstance): ErrorMessage {
  return [
    { kind: "highlighted", value: instanceToClass(type).toHumanReadable() },
  ];
}

/**
 * Generates a highlighted human-readable representation of multiple types
 *
 * E.g. `[NixInt, NixFloat, NixString]` becomes `a number, a float, or a string`
 */
export function errTypes(...types: NixTypeClass[]): ErrorMessage {
  if (types.length === 0) {
    throw new Error("classListToErrorMessage: types array is empty");
  }

  if (types.length === 1) {
    return errType(types[0]);
  }

  // Dynamically build the error message, separating with commas and "or"
  const message: ErrorMessage = [];
  for (let i = 0; i < types.length; i++) {
    if (i === types.length - 1) {
      message.push(...err`or`);
    } else if (i > 0) {
      message.push(...err`,`);
    }
    message.push(...errType(types[i]));
  }
}

export function highlighted(message: string | ErrorMessage): ErrorMessage {
  const msg = err`${message}`;
  return msg.map<HighlightedErrorMessagePart>((part) => ({
    kind: "highlighted",
    value: part.value,
  }));
}
