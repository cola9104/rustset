resource "alicloud_instance" "requested" {
  count             = var.spec.resource_count
  instance_name     = "${var.spec.ecs_name}-${count.index + 1}"
  instance_type     = var.spec.ecs_type
  image_id          = var.spec.image_id
  availability_zone = var.spec.availability_zone
  vswitch_id        = var.spec.subnet_id
  security_groups   = var.spec.security_groups
  system_disk_size  = try(var.spec.system_disk_size_gb, null)
}
output "instance_ids" { value = alicloud_instance.requested[*].id }
output "private_ips" { value = alicloud_instance.requested[*].private_ip }
output "public_ips" { value = alicloud_instance.requested[*].public_ip }
