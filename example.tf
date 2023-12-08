terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "=3.0.0"
    }
    docker = {
      source  = "kreuzwerker/docker"
      version = "=3.0.2"
    }
  }
}

provider "azurerm" {
  skip_provider_registration = true # This is only required when the User, Service Principal, or Identity running Terraform lacks the permissions to register Azure Resource Providers.
  features {}
}

provider "docker" {
  host = "unix:///var/run/docker.sock"
  registry_auth {
    address  = azurerm_container_registry.example.login_server
    username = azurerm_container_registry.example.admin_username
    password = azurerm_container_registry.example.admin_password
  }
}

resource "azurerm_resource_group" "example" {
  name     = "test-registry"
  location = "West Europe"
}

resource "azurerm_container_registry" "example" {
  name                = "containerRegistry1"
  resource_group_name = azurerm_resource_group.example.name
  location            = azurerm_resource_group.example.location
  sku                 = "Standard"
  admin_enabled       = false
}

# resource "docker_tag" "example" {
#   source_image          = "get_changed_validators"
#   target_image          = "${azurerm_container_registry.example.name}.azurecr.io/get_changed_validators"
# }

# resource "docker_image" "example" {
#   name          = docker_tag.example.target_image
# }
