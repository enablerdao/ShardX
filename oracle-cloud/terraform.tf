variable "tenancy_ocid" {}
variable "user_ocid" {}
variable "fingerprint" {}
variable "private_key_path" {}
variable "region" {}
variable "compartment_ocid" {}

provider "oci" {
  tenancy_ocid     = var.tenancy_ocid
  user_ocid        = var.user_ocid
  fingerprint      = var.fingerprint
  private_key_path = var.private_key_path
  region           = var.region
}

resource "oci_core_vcn" "shardx_vcn" {
  cidr_block     = "10.0.0.0/16"
  compartment_id = var.compartment_ocid
  display_name   = "ShardXVCN"
  dns_label      = "shardxvcn"
}

resource "oci_core_subnet" "shardx_subnet" {
  cidr_block        = "10.0.1.0/24"
  display_name      = "ShardXSubnet"
  dns_label         = "shardxsubnet"
  security_list_ids = [oci_core_security_list.shardx_security_list.id]
  compartment_id    = var.compartment_ocid
  vcn_id            = oci_core_vcn.shardx_vcn.id
  route_table_id    = oci_core_route_table.shardx_route_table.id
}

resource "oci_core_internet_gateway" "shardx_internet_gateway" {
  compartment_id = var.compartment_ocid
  display_name   = "ShardXIG"
  vcn_id         = oci_core_vcn.shardx_vcn.id
}

resource "oci_core_route_table" "shardx_route_table" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.shardx_vcn.id
  display_name   = "ShardXRouteTable"

  route_rules {
    destination       = "0.0.0.0/0"
    network_entity_id = oci_core_internet_gateway.shardx_internet_gateway.id
  }
}

resource "oci_core_security_list" "shardx_security_list" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.shardx_vcn.id
  display_name   = "ShardXSecurityList"

  egress_security_rules {
    destination = "0.0.0.0/0"
    protocol    = "all"
  }

  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"

    tcp_options {
      min = 22
      max = 22
    }
  }

  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"

    tcp_options {
      min = 54867
      max = 54867
    }
  }

  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"

    tcp_options {
      min = 54868
      max = 54868
    }
  }
}

data "oci_core_images" "ubuntu_images" {
  compartment_id           = var.compartment_ocid
  operating_system         = "Canonical Ubuntu"
  operating_system_version = "20.04"
  shape                    = "VM.Standard.E4.Flex"
  sort_by                  = "TIMECREATED"
  sort_order               = "DESC"
}

resource "oci_core_instance" "shardx_instance" {
  availability_domain = data.oci_identity_availability_domains.ads.availability_domains[0].name
  compartment_id      = var.compartment_ocid
  display_name        = "ShardXInstance"
  shape               = "VM.Standard.E4.Flex"

  shape_config {
    ocpus         = 2
    memory_in_gbs = 16
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.shardx_subnet.id
    display_name     = "PrimaryVNIC"
    assign_public_ip = true
  }

  source_details {
    source_type = "image"
    source_id   = data.oci_core_images.ubuntu_images.images[0].id
  }

  metadata = {
    ssh_authorized_keys = file("~/.ssh/id_rsa.pub")
    user_data = base64encode(<<-EOF
      #!/bin/bash
      # システムの更新
      apt-get update
      apt-get upgrade -y

      # 必要なパッケージのインストール
      apt-get install -y apt-transport-https ca-certificates curl gnupg lsb-release git

      # Dockerのインストール
      curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
      echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null
      apt-get update
      apt-get install -y docker-ce docker-ce-cli containerd.io

      # Docker Composeのインストール
      curl -L "https://github.com/docker/compose/releases/download/v2.18.1/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
      chmod +x /usr/local/bin/docker-compose

      # ShardXのクローンと起動
      git clone https://github.com/enablerdao/ShardX.git /opt/ShardX
      cd /opt/ShardX
      docker-compose up -d
      EOF
    )
  }
}

data "oci_identity_availability_domains" "ads" {
  compartment_id = var.compartment_ocid
}

output "instance_public_ip" {
  value = oci_core_instance.shardx_instance.public_ip
}

output "web_url" {
  value = "http://${oci_core_instance.shardx_instance.public_ip}:54867"
}

output "api_url" {
  value = "http://${oci_core_instance.shardx_instance.public_ip}:54868"
}