Feature: Get OIDC Config by ID

  Retrieve an OIDC configuration by its unique identifier.

  Scenario: Successfully retrieving an existing OIDC config
    Given an OIDC config exists in the read store
    When the get OIDC config by id query is executed
    Then the OIDC config should be returned

  Scenario: Returning none for a non-existent OIDC config
    When the get OIDC config by id query is executed for a non-existent id
    Then no OIDC config should be returned
