Implemented.

- Added [`/work/rtw-paired-4b06j7rq/api-envelope-baseline/src/api/invoices.py`]( /work/rtw-paired-4b06j7rq/api-envelope-baseline/src/api/invoices.py):
  - `get_invoice(repository, invoice_id)`
  - Calls `repository.find(invoice_id)`
  - Returns `ok(invoice)` when found
  - Returns `failure("invoice_not_found", "Invoice not found")` when absent

Checks run:
- `python3 -m compileall -q src/api` → passed
- `python -m compileall -q src/api` could not run because `python` is not installed in this environment (only `python3` is available)

