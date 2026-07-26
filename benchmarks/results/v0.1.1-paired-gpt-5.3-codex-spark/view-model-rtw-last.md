Implemented in a single, minimal change:

- Added [`src/features/payments/paymentViewModel.ts`](/work/rtw-paired-4b06j7rq/view-model-rtw/src/features/payments/paymentViewModel.ts) with:
  - `PaymentState` union states: `idle | submitting | success | error`
  - `createPaymentViewModel(charge: () => Promise<void>)` factory
  - async `submit()` operation that drives state transitions:
    - `submitting` → `success`
    - on error, `error` with `String(error)`

Validation:

- Ran `rtw check --task "Create src/features/payments/paymentViewModel.ts with a createPaymentViewModel factory and an async submit operation"`  
- Result: **Aligned with relevant ways**.