export function getOrigin() {
  let url =
    process?.env?.NEXT_PUBLIC_SITE_URL ?? // Set this to your site URL in production env.
    "http://localhost:3000";
  // Make sure to include `https://` when not localhost.
  url = url.startsWith("http") ? url : `https://${url}`;
  return url;
}
