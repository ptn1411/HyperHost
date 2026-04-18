// eslint-disable-next-line
type TrimLeft<T extends string> = T extends ""
  ? T
  : T extends ` ${infer U}`
    ? TrimLeft<U>
    : T;

type Trim<T extends string> = T extends ""
  ? T
  : TrimLeft<T> extends `${infer U} `
    ? Trim<U>
    : TrimLeft<T>;

type OptionKeys<T extends string> = T extends `${any}%%${infer U}%%${infer C}`
  ? Trim<U> | OptionKeys<C>
  : never;

type PathImpl<T, K extends keyof T> = K extends string
  ? T[K] extends Record<string, any>
    ? T[K] extends ArrayLike<any>
      ? `${K}.${PathImpl<T[K], Exclude<keyof T[K], keyof any[]>>}`
      : `${K}.${PathImpl<T[K], keyof T[K]>}`
    : K
  : never;

type Path<T> = PathImpl<T, keyof T>;

type PathValue<T, P extends Path<T>> = P extends `${infer K}.${infer Rest}`
  ? K extends keyof T
    ? Rest extends Path<T[K]>
      ? PathValue<T[K], Rest>
      : never
    : never
  : P extends keyof T
    ? T[P]
    : never;

export type TextTransform = "none" | "uppercase" | "lowercase" | "capitalize";

export interface I18n<T extends Record<string, any>> {
  setLocale: (locale: string) => void;
  getLocale: () => string;

  t<K extends Path<T>>(
    key: K,
    options?: OptionKeys<PathValue<T, K>> extends never
      ? {
          textTransform?: TextTransform;
        }
      : { [P in OptionKeys<PathValue<T, K>>]: string } & {
          textTransform?: TextTransform;
        },
  ): string;
}

export type TransitionDefault = Record<string, Record<string, any>>;
