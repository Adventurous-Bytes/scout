import { createServerClient } from "@supabase/ssr";
import { NextResponse, type NextRequest } from "next/server";

export interface IOptionsMiddlewareAuth {
  allowed_email_domains?: string[];
  allowed_page_paths_without_auth: string[];
  login_page_path: string;
}

export async function updateSession(
  request: NextRequest,
  options: IOptionsMiddlewareAuth,
) {
  // Validate environment variables
  const supabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL;
  const supabaseAnonKey = process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY;

  if (!supabaseUrl || !supabaseAnonKey) {
    throw new Error("Missing required Supabase environment variables");
  }

  let supabaseResponse = NextResponse.next({ request });

  const supabase = createServerClient(supabaseUrl, supabaseAnonKey, {
    cookies: {
      getAll() {
        return request.cookies.getAll();
      },
      setAll(cookiesToSet) {
        cookiesToSet.forEach(({ name, value }) =>
          request.cookies.set(name, value),
        );
        supabaseResponse = NextResponse.next({ request });
        cookiesToSet.forEach(({ name, value, options }) =>
          supabaseResponse.cookies.set(name, value, options),
        );
      },
    },
  });

  // IMPORTANT: Avoid writing any logic between createServerClient and
  // supabase.auth.getClaims(). A simple mistake could make it very hard to debug
  // issues with users being randomly logged out.

  const { data: claims, error } = await supabase.auth.getClaims();

  // Check if current path is allowed without authentication
  const isPublicPath = options.allowed_page_paths_without_auth.some((page) =>
    request.nextUrl.pathname.startsWith(page),
  );

  if (isPublicPath) {
    return supabaseResponse;
  }

  // Check authentication requirements
  const hasValidClaims = !error && claims?.claims?.sub;
  const hasValidEmail =
    claims?.claims?.email &&
    (!options.allowed_email_domains ||
      options.allowed_email_domains.some((domain) =>
        claims.claims.email!.endsWith(`@${domain}`),
      ));

  if (!hasValidClaims || (claims?.claims?.email && !hasValidEmail)) {
    // no valid claims - respond by redirecting the user to the login page
    const url = request.nextUrl.clone();
    url.pathname = options.login_page_path;
    return NextResponse.redirect(url);
  }

  // IMPORTANT: You *must* return the supabaseResponse object as it is. If you're
  // creating a new response object with NextResponse.next() make sure to:
  // 1. Pass the request in it, like so:
  //    const myNewResponse = NextResponse.next({ request })
  // 2. Copy over the cookies, like so:
  //    myNewResponse.cookies.setAll(supabaseResponse.cookies.getAll())
  // 3. Change the myNewResponse object to fit your needs, but avoid changing
  //    the cookies!
  // 4. Finally:
  //    return myNewResponse
  // If this is not done, you may be causing the browser and server to go out
  // of sync and terminate the user's session prematurely!

  return supabaseResponse;
}
