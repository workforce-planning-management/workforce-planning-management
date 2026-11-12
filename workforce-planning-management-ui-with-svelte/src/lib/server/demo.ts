// The demo (the guided tour at `/demo`): anyone can sign in as the example address and use every page
// against a separate service instance that holds only fictitious data. Server-side only.
//
// A demo session is an opaque id with the `demo:` prefix. It is never exchanged at the authentication
// service and never reaches the real service: the proxy sends it to `WPM_DEMO_API_URL` and nowhere else.
// With no `WPM_DEMO_API_URL` the demo does not exist and the prefix opens nothing.

import { WPM_DEMO_API_URL } from "./config";

/** The address that signs in to the demo. */
export const DEMO_EMAIL = "example@example.com";

const PREFIX = "demo:";

/** Whether a demo is offered by this deployment. */
export const demoAvailable = (): boolean => WPM_DEMO_API_URL !== "";

/** Whether `email` is the demo address (case and surrounding spaces do not matter). */
export const isDemoEmail = (email: string): boolean =>
  email.trim().toLowerCase() === DEMO_EMAIL;

/** A fresh demo session id. */
export const newDemoSessionId = (): string => `${PREFIX}${crypto.randomUUID()}`;

/** Whether a session id is a demo session. */
export const isDemoSession = (id: string | null | undefined): boolean =>
  typeof id === "string" && id.startsWith(PREFIX);

/** The demo user's memberships, made up from the demo service's own people: the demo has no sign-in at the
 *  authentication service, so nothing there says who the demo user is. They are an administrator of the
 *  organization the demo data belongs to, and the first permanent worker is "me".
 *
 *  To allow the guided tour to run without a database connection, we return static demo memberships
 *  instead of fetching from the API. This means the tour can work even when the demo service
 *  doesn't have a database. */
export async function demoMemberships(
  _fetchFn: typeof fetch,
  _demoApiUrl: string,
): Promise<Array<Record<string, unknown>>> {
  // Return static demo membership data that doesn't require a database connection
  // The demo user is an administrator of a sample organization
  const demoOrgRef = "demo-org";
  const demoWorkerPid = "00000000-0000-4000-8000-00000000d3a1";
  const demoPersonRef = "person:00000000-0000-4000-8000-00000000d3a1";
  const hiredOn = "2024-01-01";

  return [
    {
      pid: `00000000-0000-4000-8000-00000000d3a0`,
      person_ref: demoPersonRef,
      organization_ref: demoOrgRef,
      worker_pid: demoWorkerPid,
      employed: true,
      role: "hr_admin",
      starts_on: hiredOn,
      ends_on: null,
    },
  ];
}
