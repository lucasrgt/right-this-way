Implemented with minimal module changes:

- Added `aws_sqs_queue` resource in `[main.tf](/work/rtw-paired-4b06j7rq/terraform-tags-baseline/infra/modules/queue/main.tf:1)`:
  - `name` is sourced from `var.name`
  - `tags` follows existing module convention by merging `var.common_tags` with module-owned tags

- Added variables in `[variables.tf](/work/rtw-paired-4b06j7rq/terraform-tags-baseline/infra/modules/queue/variables.tf:1)`:
  - `name` (`string`)
  - `common_tags` (`map(string)`)

Check run:
- Attempted `terraform fmt -recursive infra/modules/queue/main.tf infra/modules/queue/variables.tf`
- Command failed in this environment because `terraform` is not installed (`command not found`).