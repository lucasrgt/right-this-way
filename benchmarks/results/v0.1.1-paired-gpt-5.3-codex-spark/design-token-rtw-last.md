Implemented.

- Added [`/work/rtw-paired-4b06j7rq/design-token-rtw/src/ui/DangerButton.tsx`](.../src/ui/DangerButton.tsx) with the same established convention as `PrimaryButton`:
  - Props: `label: string`, `disabled?: boolean`, `onClick(): void`
  - Renders a `<button>` with semantic token-based styles
  - Uses `var(--color-action-danger)` for background and existing shared tokenized text/color/spacing tokens.

- Ran the required check: `rtw check --task "Add DangerButton component with label, disabled, and onClick props"`  
  - Result: **Aligned with relevant ways.**

Ready to commit.