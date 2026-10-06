# Employee expense claims (WPM-R55)

A worker asks to be repaid for money they spent for work. They build a **claim**
from dated, categorised **items**, submit it, someone **other than the claimant**
approves or rejects it, and an approved claim is marked reimbursed. This closes the
one table-stakes gap in the benchmark scan (`.sota/last-scan.json`: frappe/hrms
`expense_claim`, OrangeHRM `orangehrmClaimPlugin`, Odoo `hr_expense`), deferred on
2026-10-05 and delivered as WPM-T97 (see [tasks.md](tasks.md)).

## WPM-R55 — Expense claims

*As an employee I can claim back what I spent; as their manager or HR I can decide
it; nobody decides their own.*

- **Claim** (`expense_claims`): title, optional description, one ISO 4217 currency
  (three upper-case letters; **mixed currencies never add**). **Statuses:**
  `draft → submitted → approved | rejected`, `approved → reimbursed`,
  `submitted → draft` (withdrawn to edit), `draft | submitted → cancelled`.
  `rejected`, `reimbursed` and `cancelled` are terminal. An illegal move is **422**
  naming the current state; two racing deciders serialize on the locked row.
- **Item** (`expense_items`): incurred date (**not in the future**), a category
  (`travel`, `accommodation`, `meals`, `equipment`, `training`, `subscriptions`,
  `other`), a positive amount in minor units (capped at 100 000 000), an optional
  description and receipt **reference** (text only — no file is stored). At most 50
  per claim. Items change **only while the claim is a draft**. The total is the
  checked sum (an overflow is an error).
- **Duplicates are flagged, not refused:** an item with the same date, category and
  amount as another on the claim, or on the claimant's other live claims, is marked
  `possible_duplicate` (two identical lunches are sometimes two lunches).
- A claim needs **at least one item to be submitted**; a **rejection needs a note**;
  `reimbursed_on` defaults to today and is **not in the future**.
- **Who may do what** (the service decides; the UI only reflects the `can` flags it
  returns):

  | | claimant | their manager | HR (in their organization) | anyone else |
  | --- | :-: | :-: | :-: | :-: |
  | see a claim | ✓ | ✓ | ✓ | — (404) |
  | build it: items, submit, withdraw, cancel | ✓ | — | ✓ (on their behalf) | — |
  | decide it: approve, reject, reimburse | **never** | ✓ | ✓ (**not their own**) | — |

  HR is the `hr_admin` role in the claimant's organization (a membership, not a policy
  attribute). A claimant who is also HR still cannot decide their own claim; their
  manager does. With auth off, every check passes (the demo default).
- **Endpoints:** `POST|GET /api/workers/{pid}/expense-claims`;
  `GET /api/expense-claims?status=` (the decision queue: the caller's direct reports'
  claims and, for HR, their organization's — **never their own**; default `submitted`);
  `GET /api/expense-claims/{pid}` (items, duplicate flags, `can`);
  `POST /api/expense-claims/{pid}/items`, `DELETE /api/expense-items/{pid}`;
  `POST /api/expense-claims/{pid}/submit|withdraw|cancel|approve|reject|reimburse`.
- **Notifications** (reference-only, closed kinds `expense_submitted`,
  `expense_decided`): the manager is told a claim awaits them; the claimant is told
  of each outcome. **No amount, title or description** is in the text.
- UI: an **Expense claims** panel on `/me` and the worker's page (hidden for anyone who
  may not see it) and an **Expense decisions** page at `/expenses`. Migration
  `m20261006_000049_expense_claims`.

## WPM-D42 — A claim is financial and personal; no one is their own approver

- **Segregation of duties is structural:** `may_decide` is false for the claimant
  whatever else they are, and the decision queue omits their own claims. This is the
  core control of any expense process, so it is a pure, exhaustively tested rule
  (`rules/expenses.rs`) and a separate enforcement test
  (`tests/enforcement_expenses.rs`) that fails if it is removed.
- **Audience:** a claim reveals where someone has been and what they spent, so it is
  the claimant's, their manager's and HR's — others get a 404, not a 403, so they
  cannot even learn it exists.
- **Quiet by default:** the audit entries (`expense_claim_created|submitted|…`) and the
  notifications carry **no amount, title or description**.
- **Erasure keeps the money, drops the words** (WPM-D22): claims are financial records
  under retention. On erasure, draft and submitted claims are **cancelled**,
  approved/reimbursed ones stay with their amounts, and every free-text field — claim
  title, description and decision note, item description and receipt reference — is
  scrubbed. Claims and items are in the subject-access export. `expense_claims` is on the
  retention sweep's soft-deleted list; items (no `deleted_at`) go with their claim.
- **A flag, not a refusal,** for duplicates; **a reason, not a guess,** for rejection.

## Not done

- **Payroll integration:** "reimbursed" is a date and an actor, not a payslip line;
  nothing pays anyone. Adding approved claims to a payroll run is the natural next step
  and needs a decision on tax treatment per category.
- **Receipts as files** (only a reference is stored); mileage and per-diem rates;
  per-category policy limits or budgets; multi-level approval; approval delegation.
- A **team view of spend** (a count of one names someone); exports to accounting.
- Currency conversion: a claim has one currency.
