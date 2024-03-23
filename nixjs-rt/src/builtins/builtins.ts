import {
  Attrset,
  EvalException,
  FALSE,
  Lambda,
  NULL,
  NixFloat,
  NixInt,
  NixList,
  NixString,
  NixType,
  TRUE,
} from "../lib";

type BuiltinsRecord = Record<string, (param: NixType) => NixType>;

export function getBuiltins() {
  const builtins: BuiltinsRecord = {
    abort: (message) => {
      throw new EvalException(
        `Evaluation aborted with the following error message: '${message.asString()}'`,
      );
    },

    add: (lhs): Lambda => {
      return new Lambda((rhs) => {
        let lhsStrict = lhs.toStrict();
        if (!(lhsStrict instanceof NixInt || lhsStrict instanceof NixFloat)) {
          throw new EvalException(
            `value is of type '${lhs.typeOf()}' while a number was expected.`,
          );
        }
        let rhsStrict = rhs.toStrict();
        if (!(rhsStrict instanceof NixInt || rhsStrict instanceof NixFloat)) {
          throw new EvalException(
            `value is of type '${rhs.typeOf()}' while a number was expected.`,
          );
        }
        return lhsStrict.add(rhsStrict);
      });
    },

    head: (list) => {
      const listStrict = list.toStrict();
      if (!(listStrict instanceof NixList)) {
        throw new EvalException(
          `Cannot apply the 'head' function on '${listStrict.typeOf()}'.`,
        );
      }
      if (listStrict.values.length === 0) {
        throw new EvalException(
          "Cannot fetch the first element in an empty list.",
        );
      }
      return listStrict.values[0];
    },

    all: (pred) => {
      const lambdaStrict = pred.toStrict();
      if (!(lambdaStrict instanceof Lambda)) {
        throw new EvalException(
          `'all' function requires another function, but got '${lambdaStrict.typeOf()}' instead.`,
        );
      }

      return new Lambda((list) => {
        const listStrict = list.toStrict();
        if (!(listStrict instanceof NixList)) {
          throw new EvalException(
            `Cannot apply the 'all' function on '${listStrict.typeOf()}'.`,
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
        throw new EvalException(
          `'any' function requires another function, but got '${lambdaStrict.typeOf()}' instead.`,
        );
      }

      return new Lambda((list) => {
        const listStrict = list.toStrict();
        if (!(listStrict instanceof NixList)) {
          throw new EvalException(
            `Cannot apply the 'any' function on '${listStrict.typeOf()}'.`,
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
        throw new EvalException(
          `Cannot apply the 'attrNames' function on '${attrsetStrict.typeOf()}'.`,
        );
      }

      const keys = Array.from(attrsetStrict.keys());
      keys.sort();

      return new NixList(keys.map((key) => new NixString(key)));
    },

    attrValues: (attrset) => {
      const attrsetStrict = attrset.toStrict();
      if (!(attrsetStrict instanceof Attrset)) {
        throw new EvalException(
          `Cannot apply the 'attrValues' function on '${attrsetStrict.typeOf()}'.`,
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
