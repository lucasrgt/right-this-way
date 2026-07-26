Implemented.

Created:

- [`/work/rtw-paired-4b06j7rq/view-model-baseline/src/features/payments/paymentViewModel.ts`]( /work/rtw-paired-4b06j7rq/view-model-baseline/src/features/payments/paymentViewModel.ts)

with:
- `PaymentState` union type mirroring the existing view-model state shape.
- `createPaymentViewModel(processPayment: () => Promise<void>)` factory.
- Async `submit()` method that sets state to `submitting`, awaits the async callback, then transitions to `success` or `error` with an error message.

Validation run:
- Attempted `tsc --noEmit`, but this environment has no TypeScript compiler installed (`No TypeScript compiler available in this environment`), so no automatic type check could be executed here.