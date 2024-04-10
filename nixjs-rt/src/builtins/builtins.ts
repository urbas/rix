import { err } from "../errors";
import { abortError } from "../errors/abort";
import { otherError } from "../errors/other";
import { typeMismatchError } from "../errors/typeError";
import {
  Attrset,
  EvalCtx,
  EvalException,
  FALSE,
  Lambda,
  NULL,
  NixFloat,
  NixInt,
  NixList,
  NixString,
  NixType,
  Path,
  TRUE,
} from "../lib";

type BuiltinsRecord = Record<string, (param: NixType) => NixType>;

export function getBuiltins() {
  const builtins: BuiltinsRecord = {
    abort: (message) => {
      throw abortError(message.asString());
    },

    import: (path) => {
      const pathStrict = path.toStrict();

      if (!(pathStrict instanceof Path || pathStrict instanceof NixString)) {
        const expected = [Path, NixString];
        throw typeMismatchError(
          pathStrict,
          expected,
          err`Import statement expected ${expected}, but got ${pathStrict}.`,
        );
      }

      const pathValue = pathStrict.toJs();

      // Below is an intrinsic function that's injected by the Nix evaluator.
      // @ts-ignore
      const resultingFn: (ctx: EvalCtx) => NixType = importNixModule(pathValue);

      const ctx = new EvalCtx(pathValue);
      return resultingFn(ctx);
    },

    add: (lhs): Lambda => {
      return new Lambda((rhs) => {
        let lhsStrict = lhs.toStrict();
        if (!(lhsStrict instanceof NixInt || lhsStrict instanceof NixFloat)) {
          let expected = [NixInt, NixFloat];
          throw typeMismatchError(
            lhsStrict,
            expected,
            err`Cannot add '${lhsStrict}', expected ${expected}.`,
          );
        }
        let rhsStrict = rhs.toStrict();
        if (!(rhsStrict instanceof NixInt || rhsStrict instanceof NixFloat)) {
          let expected = [NixInt, NixFloat];
          throw typeMismatchError(
            rhsStrict,
            expected,
            err`Cannot add '${lhsStrict}', expected ${expected}.`,
          );
        }
        return lhsStrict.add(rhsStrict);
      });
    },

    head: (list) => {
      const listStrict = list.toStrict();
      if (!(listStrict instanceof NixList)) {
        throw typeMismatchError(
          listStrict,
          NixList,
          err`Cannot apply the 'head' function on '${listStrict}', expected ${NixList}.`,
        );
      }
      if (listStrict.values.length === 0) {
        throw otherError("Cannot fetch the first element in an empty list.");
      }
      return listStrict.values[0];
    },

    all: (pred) => {
      const lambdaStrict = pred.toStrict();
      if (!(lambdaStrict instanceof Lambda)) {
        throw typeMismatchError(
          lambdaStrict,
          Lambda,
          err`'all' function requires ${Lambda}, but got ${lambdaStrict} instead.`,
        );
      }

      return new Lambda((list) => {
        const listStrict = list.toStrict();
        if (!(listStrict instanceof NixList)) {
          throw typeMismatchError(
            listStrict,
            NixList,
            err`'all' function expects ${NixList}, got '${listStrict.typeOf()}'.`,
          );
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
        throw typeMismatchError(
          lambdaStrict,
          Lambda,
          err`'any' function requires ${Lambda}, but got ${lambdaStrict} instead.`,
        );
      }

      return new Lambda((list) => {
        const listStrict = list.toStrict();
        if (!(listStrict instanceof NixList)) {
          throw typeMismatchError(
            listStrict,
            NixList,
            err`'any' function expects ${NixList}, got '${listStrict}'.`,
          );
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
        throw typeMismatchError(
          attrsetStrict,
          Attrset,
          err`'attrNames' function expects ${Attrset}, got '${attrsetStrict}'.`,
        );
      }

      const keys = Array.from(attrsetStrict.keys());
      keys.sort();

      return new NixList(keys.map((key) => new NixString(key)));
    },

    attrValues: (attrset) => {
      const attrsetStrict = attrset.toStrict();
      if (!(attrsetStrict instanceof Attrset)) {
        throw typeMismatchError(
          attrsetStrict,
          Attrset,
          err`'attrValues' function expects ${Attrset}, got '${attrsetStrict}'.`,
        );
      }

      const keys = Array.from(attrsetStrict.keys());
      keys.sort();

      return new NixList(
        keys.map((key) => attrset.select([new NixString(key)], NULL)),
      );
    },
  };

  return builtins;
}
