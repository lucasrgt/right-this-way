Created the queue module with the minimal files and convention-consistent tagging pattern:

- [infra/modules/queue/main.tf](infra/modules/queue/main.tf)
- [infra/modules/queue/variables.tf](infra/modules/queue/variables.tf)

Ran the required check:

- `rtw check --task "Create infra/modules/queue/main.tf and variables.tf for an aws_sqs_queue named from var.name"` → aligned with relevant ways.