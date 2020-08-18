export function assert<T>(
  condition: boolean,
  message = "Assertion failed",
): asserts condition {
  if (condition === false) {
    throw new Error(message)
  }
}
