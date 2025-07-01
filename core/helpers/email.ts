/**
 * Ensure email conforms to basic email format x@y.z
 * @param email
 * @returns boolean
 *
 */
export function validateEmail(email: string): boolean {
  // emails should be x@y
  const parts = email.split("@");
  if (parts.length !== 2) {
    return false;
  }
  // x should not be empty
  if (parts[0].length === 0) {
    return false;
  }
  // y should have at least one dot
  if (!parts[1].includes(".")) {
    return false;
  }
  return true;
}
