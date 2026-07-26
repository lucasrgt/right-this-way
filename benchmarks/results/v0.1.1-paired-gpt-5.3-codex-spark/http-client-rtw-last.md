Added with minimal change:

- Created [`src/clients/invoicesClient.ts`](/work/rtw-paired-4b06j7rq/http-client-rtw/src/clients/invoicesClient.ts) exporting:

```ts
export function getInvoice(baseUrl: string, invoiceId: string)
```

- The implementation mirrors `ordersClient` conventions and calls `requestWithRetry` with a `5_000` ms timeout.

Validation run:

- `rtw check --task "Add src/clients/invoicesClient.ts exporting getInvoice(baseUrl, invoiceId)"`  
  Result: `Aligned with relevant ways.`