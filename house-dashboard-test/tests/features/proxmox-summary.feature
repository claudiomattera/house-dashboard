Feature: Proxmox summary charts

    Scenario: Drawing a Proxmox summary chart
        Given the Proxmox summary configuration "proxmox/proxmox-configuration.toml"
        And the style configuration "style/light.toml"
        And the hosts "proxmox/hosts.json"
        And the statuses "proxmox/statuses.json"
        And the loads "proxmox/loads.json"
        When drawing a Proxmox summary chart
        Then the bitmap is saved to "proxmox/actual.bmp"
        Then the bitmap is the same as "proxmox/expected.bmp"

    Scenario: Drawing a dark Proxmox summary chart
        Given the Proxmox summary configuration "proxmox/proxmox-configuration.toml"
        And the style configuration "style/dark.toml"
        And the hosts "proxmox/hosts.json"
        And the statuses "proxmox/statuses.json"
        And the loads "proxmox/loads.json"
        When drawing a Proxmox summary chart
        Then the bitmap is saved to "proxmox/dark-actual.bmp"
        Then the bitmap is the same as "proxmox/dark-expected.bmp"
