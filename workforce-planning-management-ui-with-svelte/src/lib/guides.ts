// The walkthroughs (WPM-R244): for each role, the tasks in order, each with the page it starts from. The words are content
// (`guides.<role>.<task>.title` and `.body` in every locale); this file holds only the order and the addresses, so a test can
// check that every task names a page the app has and every key exists.

/** The roles a walkthrough is written for, in the order they are offered. */
export const GUIDE_ROLES = ["employee", "manager", "hr"] as const;
export type GuideRole = (typeof GUIDE_ROLES)[number];

/** One task: its key under `guides.<role>.` and the page (without the locale) where it starts. */
export interface GuideTask {
  id: string;
  path: string;
}

export const GUIDES: Record<GuideRole, readonly GuideTask[]> = {
  employee: [
    { id: "required", path: "/me/required" },
    { id: "waiting", path: "/me" },
    { id: "goal", path: "/me/goals" },
    { id: "qualification", path: "/me" },
    { id: "payslips", path: "/me/payslips" },
    { id: "feedback", path: "/me" },
  ],
  manager: [
    { id: "team", path: "/me" },
    { id: "goals", path: "/me/goals" },
    { id: "padr", path: "/me/padr" },
    { id: "appraisal", path: "/workers" },
  ],
  hr: [
    { id: "requirement", path: "/mandatory-learning" },
    { id: "completion", path: "/mandatory-learning" },
    { id: "verify", path: "/qualifications" },
    { id: "vacancy", path: "/requisitions" },
    { id: "movement", path: "/movements" },
  ],
};

/** Whether `role` names a walkthrough. */
export function isGuideRole(role: string): role is GuideRole {
  return (GUIDE_ROLES as readonly string[]).includes(role);
}
