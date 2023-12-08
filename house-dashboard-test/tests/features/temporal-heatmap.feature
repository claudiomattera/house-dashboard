Feature: Temporal heatmap charts

    Background:

    Scenario: Drawing a temporal-heatmap chart
        Given the temporal heatmap configuration "temporal-heatmap/outdoor-temperature-configuration.toml"
        And the style configuration "style/light.toml"
        And the time series "temporal-heatmap/outdoor-temperature.json"
        When drawing a temporal heatmap chart
        Then the bitmap is the same as "temporal-heatmap/outdoor-temperature-expected.bmp"
        # Then the bitmap is saved to "actual.bmp"

    Scenario: Drawing a dark temporal-heatmap chart
        Given the temporal heatmap configuration "temporal-heatmap/outdoor-temperature-configuration.toml"
        And the style configuration "style/dark.toml"
        And the time series "temporal-heatmap/outdoor-temperature.json"
        When drawing a temporal heatmap chart
        Then the bitmap is the same as "temporal-heatmap/outdoor-temperature-dark-expected.bmp"
        # Then the bitmap is saved to "actual.bmp"
