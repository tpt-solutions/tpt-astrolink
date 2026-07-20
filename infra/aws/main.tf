# Copyright 2026 TPT Solutions. Licensed MIT OR Apache-2.0.

# AWS provisioning for TPT AstroLink — Cloud Core backing services.
#
# Provisions the network fabric, S3 bucket for FITS storage, Aurora PostgreSQL
# (RDS) for metadata, and Secrets Manager entries consumed by Cloud Core.
#
# Usage:
#   terraform init
#   terraform plan  -var="project=tpt-astrolink" -var="env=prod"
#   terraform apply -var="project=tpt-astrolink" -var="env=prod"
#
# NOTE: secrets are sourced from environment variables / a tfvars file that is
# NEVER committed. The RDS master password below is random and stored only in
# Secrets Manager.

terraform {
  required_version = ">= 1.5.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.region
}

variable "project" {
  type    = string
  default = "tpt-astrolink"
}

variable "env" {
  type    = string
  default = "prod"
}

variable "region" {
  type    = string
  default = "ap-southeast-2"
}

variable "vpc_cidr" {
  type    = string
  default = "10.42.0.0/16"
}

variable "db_instance_class" {
  type    = string
  default = "db.t4g.medium"
}

variable "db_allocated_storage" {
  type    = number
  default = 100
}

variable "db_master_username" {
  type      = string
  default   = "tptadmin"
  sensitive = true
}

# Provided out-of-band (CI secret / tfvars). Never hardcode a real password.
variable "db_master_password" {
  type      = string
  sensitive = true
}

locals {
  name = "${var.project}-${var.env}"
  common_tags = {
    Project   = var.project
    Env       = var.env
    Owner     = "TPT Solutions"
    ManagedBy = "terraform"
  }
}

# ---------------------------------------------------------------------------
# Networking
# ---------------------------------------------------------------------------
resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_support   = true
  enable_dns_hostnames = true
  tags                 = merge(local.common_tags, { Name = "${local.name}-vpc" })
}

data "aws_availability_zones" "available" {
  state = "available"
}

resource "aws_subnet" "private" {
  count             = 2
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index)
  availability_zone = data.aws_availability_zones.available.names[count.index]
  tags              = merge(local.common_tags, { Name = "${local.name}-private-${count.index}" })
}

resource "aws_subnet" "public" {
  count                   = 2
  vpc_id                  = aws_vpc.main.id
  cidr_block              = cidrsubnet(var.vpc_cidr, 8, 100 + count.index)
  availability_zone       = data.aws_availability_zones.available.names[count.index]
  map_public_ip_on_launch = true
  tags                    = merge(local.common_tags, { Name = "${local.name}-public-${count.index}" })
}

resource "aws_internet_gateway" "igw" {
  vpc_id = aws_vpc.main.id
  tags   = merge(local.common_tags, { Name = "${local.name}-igw" })
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.igw.id
  }
  tags = merge(local.common_tags, { Name = "${local.name}-public-rt" })
}

resource "aws_route_table_association" "public" {
  count          = length(aws_subnet.public)
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

# ---------------------------------------------------------------------------
# S3 — FITS storage (docs/storage/s3-layout.md)
# ---------------------------------------------------------------------------
resource "aws_s3_bucket" "fits" {
  bucket = "${local.name}-fits"
  tags   = merge(local.common_tags, { Name = "${local.name}-fits" })
}

resource "aws_s3_bucket_versioning" "fits" {
  bucket = aws_s3_bucket.fits.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "fits" {
  bucket = aws_s3_bucket.fits.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "aws:kms"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "fits" {
  bucket                  = aws_s3_bucket.fits.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# Lifecycle: move cold FITS to Glacier after 90 days.
resource "aws_s3_bucket_lifecycle_configuration" "fits" {
  bucket = aws_s3_bucket.fits.id
  rule {
    id     = "glacier-transition"
    status = "Enabled"
    transition {
      days          = 90
      storage_class = "GLACIER"
    }
    expiration {
      days = 3650
    }
  }
}

# Deny any non-TLS (plaintext HTTP) access to the FITS bucket.
resource "aws_s3_bucket_policy" "fits_tls" {
  bucket = aws_s3_bucket.fits.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid       = "DenyInsecureTransport"
        Effect    = "Deny"
        Principal = "*"
        Action    = "s3:*"
        Resource = [
          aws_s3_bucket.fits.arn,
          "${aws_s3_bucket.fits.arn}/*",
        ]
        Condition = {
          Bool = {
            "aws:SecureTransport" = "false"
          }
        }
      }
    ]
  })
}

# ---------------------------------------------------------------------------
# RDS Aurora PostgreSQL — metadata store
# ---------------------------------------------------------------------------
resource "aws_db_subnet_group" "main" {
  name       = "${local.name}-db-subnet"
  subnet_ids = aws_subnet.private[*].id
  tags       = local.common_tags
}

resource "aws_security_group" "db" {
  name        = "${local.name}-db-sg"
  description = "Allow Postgres from within the VPC"
  vpc_id      = aws_vpc.main.id
  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }
  tags = local.common_tags
}

resource "aws_rds_cluster" "postgres" {
  cluster_identifier      = "${local.name}-pg"
  engine                  = "aurora-postgresql"
  engine_version          = "16.4"
  database_name           = "tpt"
  master_username         = var.db_master_username
  master_password         = var.db_master_password
  db_subnet_group_name    = aws_db_subnet_group.main.name
  vpc_security_group_ids  = [aws_security_group.db.id]
  storage_encrypted       = true
  backup_retention_period = 14
  deletion_protection     = true
  skip_final_snapshot     = false
  final_snapshot_identifier = "${local.name}-pg-final"
  tags                    = local.common_tags
}

resource "aws_rds_cluster_instance" "postgres" {
  count               = 1
  identifier          = "${local.name}-pg-1"
  cluster_identifier  = aws_rds_cluster.postgres.id
  instance_class      = var.db_instance_class
  engine              = aws_rds_cluster.postgres.engine
  engine_version      = aws_rds_cluster.postgres.engine_version
  publicly_accessible = false
}

# ---------------------------------------------------------------------------
# Secrets Manager — connection material for Cloud Core
# ---------------------------------------------------------------------------
resource "aws_secretsmanager_secret" "db" {
  name = "${local.name}/rds/postgres"
  tags = local.common_tags
}

resource "aws_secretsmanager_secret_version" "db" {
  secret_id = aws_secretsmanager_secret.db.id
  secret_string = jsonencode({
    host     = aws_rds_cluster.postgres.endpoint
    port     = 5432
    dbname   = aws_rds_cluster.postgres.database_name
    username = aws_rds_cluster.postgres.master_username
    password = var.db_master_password
  })
}

resource "aws_secretsmanager_secret" "s3" {
  name = "${local.name}/s3/fits"
  tags = local.common_tags
}

resource "aws_secretsmanager_secret_version" "s3" {
  secret_id = aws_secretsmanager_secret.s3.id
  secret_string = jsonencode({
    bucket = aws_s3_bucket.fits.bucket
    region = var.region
  })
}

# ---------------------------------------------------------------------------
# Outputs
# ---------------------------------------------------------------------------
output "vpc_id" {
  value = aws_vpc.main.id
}

output "fits_bucket" {
  value = aws_s3_bucket.fits.bucket
}

output "db_endpoint" {
  value = aws_rds_cluster.postgres.endpoint
}

output "db_secret_arn" {
  value = aws_secretsmanager_secret.db.arn
}

output "s3_secret_arn" {
  value = aws_secretsmanager_secret.s3.arn
}
