Implemented.

- Added [`src/api/invoices.py`](/work/rtw-paired-4b06j7rq/api-envelope-rtw/src/api/invoices.py) with:
  - `get_invoice(repository, invoice_id)`
  - repository lookup via `repository.find(invoice_id)`
  - `ok(invoice)` on success
  - `failure("invoice_not_found", "Invoice not found")` when missing

Validation run:
- `rtw check --task "Add src/api/invoices.py with get_invoice(repository, invoice_id), returning the repository invoice or a not-found response"` → passed (`Aligned with relevant ways.`)

Ready to commit.