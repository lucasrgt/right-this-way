Implemented with the smallest change by adding a new component:

- [src/ui/DangerButton.tsx](/work/rtw-paired-4b06j7rq/design-token-baseline/src/ui/DangerButton.tsx): renders a button with `label`, optional `disabled`, and `onClick` props, matching the existing `PrimaryButton` structure and inline style convention (danger token background + shared action text/padding).

Checks run:
- Attempted `npm run` to run the project’s standard checks.
- Result: failed because the repository has no `package.json` at the root (`npm` reports `ENOENT`), so no lint/build/test command is available in this workspace.

Working tree is clean aside from the new file:
- `src/ui/DangerButton.tsx` is the only uncommitted addition.