Implemented.

- Added [`src/clients/invoicesClient.ts`]( /work/rtw-paired-4b06j7rq/http-client-baseline/src/clients/invoicesClient.ts) with:

```ts
import { requestWithRetry } from "./request";

export function getInvoice(baseUrl: string, invoiceId: string) {
  return requestWithRetry(`${baseUrl}/invoices/${invoiceId}`, {
    timeoutMs: 5_000,
  });
}
```

- Kept behavior consistent with [`ordersClient.ts`]( /work/rtw-paired-4b06j7rq/http-client-baseline/src/clients/ordersClient.ts): same request utility, path style, timeout, and export pattern.

Check run:
- `tsc --noEmit` could not run because TypeScript tooling is not configured in this environment.