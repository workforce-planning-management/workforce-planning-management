// The shape of an Idempotency-Key the proxy will forward (WPM-R132).

/** Letters, digits, hyphen and underscore, 16 to 64 characters. */
const KEY = /^[A-Za-z0-9_-]{16,64}$/;

/** Whether `value` is a key the proxy will pass on to the API. */
export function isIdempotencyKey(value: string): boolean {
  return KEY.test(value);
}
