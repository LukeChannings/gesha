export function assert<T>(
  condition: boolean,
  message = "Assertion failed",
): asserts condition {
  if (condition === false) {
    throw new Error(message)
  }
}

export function isRecord(data: unknown): data is Record<string, unknown> {
  return typeof data === "object" && data !== null
}
