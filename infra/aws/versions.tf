# Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.
# Terraform backend config and provider requirements.

terraform {
  backend "s3" {
    # Configure per-environment via -backend-config or a backend.hcl file.
    # key            = "tpt-astrolink/prod/terraform.tfstate"
    # bucket         = "<state-bucket>"
    # dynamodb_table = "<state-lock-table>"
    # region         = "ap-southeast-2"
    encrypt = true
  }
}
