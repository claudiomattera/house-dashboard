Feature: Geographical heatmap charts

    Scenario: Drawing a geographical-heatmap chart of Italy population
        Given the geographical heatmap configuration "geographical-heatmap/italy-configuration.toml"
        And the style configuration "style/light.toml"
        And the values mapping "geographical-heatmap/italy.json"
        When drawing a geographical heatmap chart
        Then the bitmap is saved to "geographical-heatmap/italy-actual.bmp"
        Then the bitmap is the same as "geographical-heatmap/italy-expected.bmp"

    Scenario: Drawing a geographical-heatmap chart of apartment temperature
        Given the geographical heatmap configuration "geographical-heatmap/apartment-configuration.toml"
        And the style configuration "style/light.toml"
        And the values mapping "geographical-heatmap/apartment.json"
        When drawing a geographical heatmap chart
        Then the bitmap is saved to "geographical-heatmap/apartment-actual.bmp"
        Then the bitmap is the same as "geographical-heatmap/apartment-expected.bmp"

    Scenario: Drawing a dark geographical-heatmap chart of Italy population
        Given the geographical heatmap configuration "geographical-heatmap/italy-dark-configuration.toml"
        And the style configuration "style/dark.toml"
        And the values mapping "geographical-heatmap/italy.json"
        When drawing a geographical heatmap chart
        Then the bitmap is saved to "geographical-heatmap/italy-dark-actual.bmp"
        Then the bitmap is the same as "geographical-heatmap/italy-dark-expected.bmp"

    Scenario: Drawing a dark geographical-heatmap chart of apartment temperature
        Given the geographical heatmap configuration "geographical-heatmap/apartment-dark-configuration.toml"
        And the style configuration "style/dark.toml"
        And the values mapping "geographical-heatmap/apartment.json"
        When drawing a geographical heatmap chart
        Then the bitmap is saved to "geographical-heatmap/apartment-dark-actual.bmp"
        Then the bitmap is the same as "geographical-heatmap/apartment-dark-expected.bmp"
