resource "huaweicloud_compute_instance" "requested" {
  count              = var.spec.resource_count
  name               = "${var.spec.ecs_name}-${count.index + 1}"
  flavor_id          = var.spec.ecs_type
  image_id           = var.spec.image_id
  availability_zone  = var.spec.availability_zone
  security_group_ids = var.spec.security_groups
  system_disk_size   = try(var.spec.system_disk_size_gb, null)
  network {
    uuid = var.spec.subnet_id
  }
}
output "instance_ids" { value = huaweicloud_compute_instance.requested[*].id }
output "private_ips" { value = huaweicloud_compute_instance.requested[*].access_ip_v4 }
