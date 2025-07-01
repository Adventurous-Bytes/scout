import { validateEmail } from "./email";

const APPROVED_DOMAINS = ["adventurelabs.earth", "conservaition.ai"];

export function isEmailValidForLogin(
  email: string,
  approved_domains: string[] = APPROVED_DOMAINS
): boolean {
  return (
    validateEmail(email) && isEmailFromApprovedDomain(email, approved_domains)
  );
}

function isEmailFromApprovedDomain(
  email: string,
  approved_domains: string[] = APPROVED_DOMAINS
): boolean {
  return approved_domains.filter((domain) => email.endsWith(domain)).length > 0;
}
