import { err, errType, errTypes, highlighted } from "./errors";
import { abortError } from "./errors/abort";
import { otherError } from "./errors/other";
import { typeMismatchError } from "./errors/typeError";
import {
  Attrset,
  EvalCtx,
  FALSE,
  Lambda,
  NULL,
  NixBool,
  NixFloat,
  NixInt,
  NixList,
  NixNull,
  NixString,
  NixType,
  NixTypeClass,
  Path,
  TRUE,
  nixBoolFromJs,
} from "./lib";
import { dirOf, isAbsolutePath, normalizePath } from "./utils";

type BuiltinsRecord = Record<string, (param: NixType) => NixType>;

function builtinBasicTypeMismatchError(
  fnName: string,
  got: NixType,
  expects: NixTypeClass | NixTypeClass[],
) {
  if (!Array.isArray(expects)) {
    expects = [expects];
  }

  return typeMismatchError(
    got,
    expects,
    err`${fnName} expects ${errTypes(...expects)}, got ${errType(got)}.`,
  );
}

export function getBuiltins() {
  // Builtins are sorted by the order they appear in the Nix manual
  // https://nixos.org/manual/nix/stable/language/builtins.html

  const builtins: BuiltinsRecord = {
    derivation: (arg) => {
      throw new Error("unimplemented");
    },

    abort: (message) => {
      throw abortError(message.asString());
    },

    add: (lhs) => {
      return new Lambda((rhs) => {
        let lhsStrict = lhs.toStrict();
        if (!(lhsStrict instanceof NixInt || lhsStrict instanceof NixFloat)) {
          let expected = [NixInt, NixFloat];
          throw builtinBasicTypeMismatchError("add", lhsStrict, expected);
        }
        let rhsStrict = rhs.toStrict();
        if (!(rhsStrict instanceof NixInt || rhsStrict instanceof NixFloat)) {
          let expected = [NixInt, NixFloat];
          throw builtinBasicTypeMismatchError("add", rhsStrict, expected);
        }
        return lhsStrict.add(rhsStrict);
      });
    },

    addDrvOutputDependencies: (arg) => {
      throw new Error("unimplemented");
    },

    all: (pred) => {
      const lambdaStrict = pred.toStrict();
      if (!(lambdaStrict instanceof Lambda)) {
        throw builtinBasicTypeMismatchError("all", lambdaStrict, Lambda);
      }

      return new Lambda((list) => {
        const listStrict = list.toStrict();
        if (!(listStrict instanceof NixList)) {
          throw builtinBasicTypeMismatchError("all", listStrict, NixList);
        }

        for (const element of listStrict.values) {
          const result = lambdaStrict.apply(element);
          if (!result.asBoolean()) {
            return FALSE;
          }
        }

        return TRUE;
      });
    },

    any: (pred) => {
      const lambdaStrict = pred.toStrict();
      if (!(lambdaStrict instanceof Lambda)) {
        throw builtinBasicTypeMismatchError("any", lambdaStrict, Lambda);
      }

      return new Lambda((list) => {
        const listStrict = list.toStrict();
        if (!(listStrict instanceof NixList)) {
          throw builtinBasicTypeMismatchError("any", listStrict, NixList);
        }

        for (const element of listStrict.values) {
          const result = lambdaStrict.apply(element);
          if (result.asBoolean()) {
            return TRUE;
          }
        }

        return FALSE;
      });
    },

    attrNames: (attrset) => {
      const attrsetStrict = attrset.toStrict();
      if (!(attrsetStrict instanceof Attrset)) {
        throw builtinBasicTypeMismatchError(
          "attrNames",
          attrsetStrict,
          Attrset,
        );
      }

      const keys = Array.from(attrsetStrict.keys());
      keys.sort();

      return new NixList(keys.map((key) => new NixString(key)));
    },

    attrValues: (attrset) => {
      const attrsetStrict = attrset.toStrict();
      if (!(attrsetStrict instanceof Attrset)) {
        throw builtinBasicTypeMismatchError(
          "attrValues",
          attrsetStrict,
          Attrset,
        );
      }

      const keys = Array.from(attrsetStrict.keys());
      keys.sort();

      return new NixList(
        keys.map((key) => attrset.select([new NixString(key)], NULL)),
      );
    },

    baseNameOf: (path) => {
      // Can take a string or path
      const pathStrict = path.toStrict();
      if (!(pathStrict instanceof Path || pathStrict instanceof NixString)) {
        const expected = [Path, NixString];
        throw builtinBasicTypeMismatchError("baseNameOf", pathStrict, expected);
      }

      let pathValue = pathStrict.toJs();
      if (pathValue.endsWith("/")) {
        pathValue = pathValue.slice(0, -1);
      }

      const parts = pathValue.split("/");
      return new NixString(parts[parts.length - 1]);
    },

    bitAnd: (arg) => {
      throw new Error("unimplemented");
    },

    bitOr: (arg) => {
      throw new Error("unimplemented");
    },

    bitXor: (arg) => {
      throw new Error("unimplemented");
    },

    break: (arg) => {
      throw new Error("unimplemented");
    },

    catAttrs: (arg) => {
      throw new Error("unimplemented");
    },

    ceil: (arg) => {
      throw new Error("unimplemented");
    },

    compareVersions: (arg) => {
      throw new Error("unimplemented");
    },

    concatLists: (arg) => {
      throw new Error("unimplemented");
    },

    concatMap: (arg) => {
      throw new Error("unimplemented");
    },

    concatStringsSep: (arg) => {
      throw new Error("unimplemented");
    },

    convertHash: (arg) => {
      throw new Error("unimplemented");
    },

    deepSeq: (arg) => {
      throw new Error("unimplemented");
    },

    dirOf: (arg) => {
      throw new Error("unimplemented");
    },

    div: (arg) => {
      throw new Error("unimplemented");
    },

    elem: (arg) => {
      throw new Error("unimplemented");
    },

    elemAt: (arg) => {
      throw new Error("unimplemented");
    },

    fetchClosure: (arg) => {
      throw new Error("unimplemented");
    },

    fetchGit: (arg) => {
      throw new Error("unimplemented");
    },

    fetchTarball: (arg) => {
      throw new Error("unimplemented");
    },

    fetchTree: (arg) => {
      throw new Error("unimplemented");
    },

    fetchurl: (arg) => {
      throw new Error("unimplemented");
    },

    filter: (arg) => {
      throw new Error("unimplemented");
    },

    filterSource: (arg) => {
      throw new Error("unimplemented");
    },

    findFile: (arg) => {
      throw new Error("unimplemented");
    },

    flakeRefToString: (arg) => {
      throw new Error("unimplemented");
    },

    floor: (arg) => {
      throw new Error("unimplemented");
    },

    foldl: (arg) => {
      throw new Error("unimplemented");
    },

    fromJSON: (arg) => {
      throw new Error("unimplemented");
    },

    fromTOML: (arg) => {
      throw new Error("unimplemented");
    },

    functionArgs: (arg) => {
      throw new Error("unimplemented");
    },

    genList: (arg) => {
      throw new Error("unimplemented");
    },

    genericClosure: (arg) => {
      throw new Error("unimplemented");
    },

    getAttr: (arg) => {
      throw new Error("unimplemented");
    },

    getContext: (arg) => {
      throw new Error("unimplemented");
    },

    getEnv: (arg) => {
      throw new Error("unimplemented");
    },

    getFlake: (arg) => {
      throw new Error("unimplemented");
    },

    groupBy: (arg) => {
      throw new Error("unimplemented");
    },

    hasAttr: (arg) => {
      throw new Error("unimplemented");
    },

    hasContext: (arg) => {
      throw new Error("unimplemented");
    },

    hashFile: (arg) => {
      throw new Error("unimplemented");
    },

    hashString: (arg) => {
      throw new Error("unimplemented");
    },

    head: (list) => {
      const listStrict = list.toStrict();
      if (!(listStrict instanceof NixList)) {
        throw typeMismatchError(
          listStrict,
          NixList,
          err`Cannot apply the 'head' function on '${errType(listStrict)}', expected ${errType(NixList)}.`,
        );
      }
      if (listStrict.values.length === 0) {
        throw otherError(
          "Cannot fetch the first element in an empty list.",
          "builtins-head-on-empty-list",
        );
      }
      return listStrict.values[0];
    },

    import: (path) => {
      const pathStrict = path.toStrict();

      if (!(pathStrict instanceof Path || pathStrict instanceof NixString)) {
        const expected = [Path, NixString];
        throw builtinBasicTypeMismatchError("import", pathStrict, expected);
      }

      let pathValue = "";
      if (pathStrict instanceof NixString) {
        pathValue = normalizePath(pathStrict.toJs());
      } else if (pathStrict instanceof Path) {
        pathValue = pathStrict.toJs();
      }

      // Check if it's an absolute path. Relative paths are not allowed.
      // Path data types are always automatically absolute.
      if (!isAbsolutePath(pathValue)) {
        throw otherError(
          err`string ${highlighted(JSON.stringify(pathValue))} doesn't represent an absolute path. Only absolute paths are allowed for imports.`,
          "builtins-import-non-absolute-path",
        );
      }

      const resultingFn = importNixModule(pathValue);

      const newCtx = new EvalCtx(dirOf(pathValue));
      return resultingFn(newCtx);
    },

    intersectAttrs: (arg) => {
      throw new Error("unimplemented");
    },

    isAttrs: (arg) => {
      return nixBoolFromJs(arg instanceof Attrset);
    },

    isBool: (arg) => {
      return nixBoolFromJs(arg instanceof NixBool);
    },

    isFloat: (arg) => {
      return nixBoolFromJs(arg instanceof NixFloat);
    },

    isFunction: (arg) => {
      return nixBoolFromJs(arg instanceof Lambda);
    },

    isInt: (arg) => {
      return nixBoolFromJs(arg instanceof NixInt);
    },

    isList: (arg) => {
      return nixBoolFromJs(arg instanceof NixList);
    },

    isNull: (arg) => {
      return nixBoolFromJs(arg instanceof NixNull);
    },

    isPath: (arg) => {
      return nixBoolFromJs(arg instanceof Path);
    },

    isString: (arg) => {
      return nixBoolFromJs(arg instanceof NixString);
    },

    length: (arg) => {
      throw new Error("unimplemented");
    },

    lessThan: (arg) => {
      throw new Error("unimplemented");
    },

    listToAttrs: (arg) => {
      throw new Error("unimplemented");
    },

    map: (arg) => {
      throw new Error("unimplemented");
    },

    mapAttrs: (arg) => {
      throw new Error("unimplemented");
    },

    match: (arg) => {
      throw new Error("unimplemented");
    },

    mul: (arg) => {
      throw new Error("unimplemented");
    },

    outputOf: (arg) => {
      throw new Error("unimplemented");
    },

    parseDrvName: (arg) => {
      throw new Error("unimplemented");
    },

    parseFlakeRef: (arg) => {
      throw new Error("unimplemented");
    },

    partition: (arg) => {
      throw new Error("unimplemented");
    },

    path: (arg) => {
      throw new Error("unimplemented");
    },

    pathExists: (arg) => {
      throw new Error("unimplemented");
    },

    placeholder: (arg) => {
      throw new Error("unimplemented");
    },

    readDir: (arg) => {
      throw new Error("unimplemented");
    },

    readFile: (arg) => {
      throw new Error("unimplemented");
    },

    readFileType: (arg) => {
      throw new Error("unimplemented");
    },

    removeAttrs: (arg) => {
      throw new Error("unimplemented");
    },

    replaceStrings: (arg) => {
      throw new Error("unimplemented");
    },

    seq: (arg) => {
      throw new Error("unimplemented");
    },

    sort: (arg) => {
      throw new Error("unimplemented");
    },

    split: (arg) => {
      throw new Error("unimplemented");
    },

    splitVersion: (arg) => {
      throw new Error("unimplemented");
    },

    storePath: (arg) => {
      throw new Error("unimplemented");
    },

    stringLength: (arg) => {
      throw new Error("unimplemented");
    },

    sub: (arg) => {
      throw new Error("unimplemented");
    },

    substring: (arg) => {
      throw new Error("unimplemented");
    },

    tail: (arg) => {
      throw new Error("unimplemented");
    },

    throw: (arg) => {
      throw new Error("unimplemented");
    },

    toFile: (arg) => {
      throw new Error("unimplemented");
    },

    toJSON: (arg) => {
      throw new Error("unimplemented");
    },

    toPath: (arg) => {
      throw new Error("unimplemented");
    },

    toString: (arg) => {
      throw new Error("unimplemented");
    },

    toXML: (arg) => {
      throw new Error("unimplemented");
    },

    trace: (arg) => {
      throw new Error("unimplemented");
    },

    traceVerbose: (arg) => {
      throw new Error("unimplemented");
    },

    tryEval: (arg) => {
      throw new Error("unimplemented");
    },

    typeOf: (arg) => {
      throw new Error("unimplemented");
    },

    unsafeDiscardOutputDependency: (arg) => {
      throw new Error("unimplemented");
    },

    zipAttrsWith: (arg) => {
      throw new Error("unimplemented");
    },
  };

  return builtins;
}
