Feature: Infrastructure summary charts

    Scenario: Drawing an infrastructure summary chart
        Given the infrastructure summary configuration "infrastructure/infrastructure-configuration.toml"
        And the style configuration "style/light.toml"
        And the hosts "infrastructure/hosts.json"
        And the loads "infrastructure/loads.json"
        And the current time "2023-12-08T19:02:00Z"
        When drawing an infrastructure summary chart
        Then the bitmap is saved to "infrastructure/actual.bmp"
        Then the bitmap is the same as "infrastructure/expected.bmp"

    Scenario: Drawing a dark infrastructure summary chart
        Given the infrastructure summary configuration "infrastructure/infrastructure-configuration.toml"
        And the style configuration "style/dark.toml"
        And the hosts "infrastructure/hosts.json"
        And the loads "infrastructure/loads.json"
        And the current time "2023-12-08T19:02:00Z"
        When drawing an infrastructure summary chart
        Then the bitmap is saved to "infrastructure/dark-actual.bmp"
        Then the bitmap is the same as "infrastructure/dark-expected.bmp"
