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
 *  organization the demo data belongs to, and the first permanent worker is "me". */
export async function demoMemberships(
  fetchFn: typeof fetch,
  demoApiUrl: string,
): Promise<Array<Record<string, unknown>>> {
  const response = await fetchFn(`${demoApiUrl}/api/workers`);
  if (!response.ok) return [];
  const people = (await response.json()) as Array<{
    pid: string;
    organization_ref: string;
    employment_type: string;
    status: string;
    hired_on: string;
  }>;
  const me =
    people.find(
      (w) => w.employment_type === "permanent" && w.status === "active",
    ) ?? people[0];
  if (!me) return [];
  // The demo user is an administrator of every organization in the demo: those that have people, and the
  // parents that contain them (a membership in a parent reads as one in every organization beneath it).
  const parents: string[] = [];
  for (const child of new Set(people.map((w) => w.organization_ref))) {
    const edges = await fetchFn(
      `${demoApiUrl}/api/organization-confederations?child_organization_ref=${encodeURIComponent(child)}`,
    );
    if (!edges.ok) continue;
    for (const e of (await edges.json()) as Array<{
      parent_organization_ref?: string;
    }>) {
      if (typeof e.parent_organization_ref === "string")
        parents.push(e.parent_organization_ref);
    }
  }
  const others = [
    ...new Set([...people.map((w) => w.organization_ref), ...parents]),
  ].filter((ref) => ref !== me.organization_ref);
  const make = (ref: string, n: number, worker: string | null) => ({
    pid: `00000000-0000-4000-8000-00000000d3${(0xa0 + n).toString(16)}`,
    person_ref: "person:00000000-0000-4000-8000-00000000d3a1",
    organization_ref: ref,
    worker_pid: worker,
    employed: worker !== null,
    role: "hr_admin",
    starts_on: me.hired_on,
    ends_on: null,
  });
  return [
    make(me.organization_ref, 0, me.pid),
    ...others.map((ref, i) => make(ref, i + 1, null)),
  ];
}
