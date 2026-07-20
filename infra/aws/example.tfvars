project             = "tpt-astrolink"
env                 = "prod"
region              = "ap-southeast-2"
vpc_cidr            = "10.42.0.0/16"
db_instance_class   = "db.t4g.medium"
db_allocated_storage = 100
db_master_username  = "tptadmin"
# db_master_password MUST be supplied at apply time, never stored in this file.
