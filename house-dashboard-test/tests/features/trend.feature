Feature: Trend charts

    Scenario: Drawing a trend chart
        Given the trend configuration "trend/room-temperature-configuration.toml"
        And the style configuration "style/light.toml"
        And the data range "2020-09-29T00:00:00Z" to "2020-09-30T00:00:00Z"
        And the time series mapping "trend/room-temperature-mapping.json"
        When drawing a trend chart
        Then the bitmap is saved to "trend/room-temperature-actual.bmp"
        Then the bitmap is the same as "trend/room-temperature-expected.bmp"

    Scenario: Drawing a dark trend chart
        Given the trend configuration "trend/room-temperature-configuration.toml"
        And the style configuration "style/dark.toml"
        And the data range "2020-09-29T00:00:00Z" to "2020-09-30T00:00:00Z"
        And the time series mapping "trend/room-temperature-mapping.json"
        When drawing a trend chart
        Then the bitmap is saved to "trend/room-temperature-dark-actual.bmp"
        Then the bitmap is the same as "trend/room-temperature-dark-expected.bmp"
