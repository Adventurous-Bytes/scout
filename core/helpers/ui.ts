import { Role, IUser } from "../types/db";

export function formatRoleCasing(role: Role) {
  return role.charAt(0).toUpperCase() + role.slice(1);
}

export function formatUsernameInitials(username: string) {
  if (!username) {
    return "";
  }
  return username.slice(0, 2).toUpperCase();
}

export function roundToXDecimals(num: number, decimals: number) {
  return Math.round(num * Math.pow(10, decimals)) / Math.pow(10, decimals);
}
