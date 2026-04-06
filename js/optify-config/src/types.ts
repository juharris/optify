/** Any object with a parse method, compatible with Zod schemas. */
export interface TypeSchema<T> {
  parse(data: unknown): T;
}
