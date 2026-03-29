Feature: OIDC Configuration Management

  Scenario: Successfully creating an OIDC config
    Given an organization exists
    When an OIDC config is created with issuer "https://auth.example.com" and audience "pigeon-api"
    Then the OIDC config should be created successfully
    And it should have the correct issuer and audience

  Scenario: Rejecting an OIDC config with empty issuer
    Given an organization exists
    When an OIDC config is created with issuer "" and audience "pigeon-api"
    Then the OIDC config creation should fail with a validation error

  Scenario: Rejecting an OIDC config with whitespace-only issuer
    Given an organization exists
    When an OIDC config is created with issuer "   " and audience "pigeon-api"
    Then the OIDC config creation should fail with a validation error

  Scenario: Rejecting an OIDC config with empty audience
    Given an organization exists
    When an OIDC config is created with issuer "https://auth.example.com" and audience ""
    Then the OIDC config creation should fail with a validation error

  Scenario: Deleting an OIDC config when multiple exist
    Given an organization with multiple OIDC configs exists
    When the OIDC config is deleted
    Then the OIDC config should be removed

  Scenario: Rejecting deletion of the last OIDC config
    Given an organization with an OIDC config exists
    When the OIDC config is deleted
    Then the deletion should fail with a validation error

  Scenario: Deleting a non-existent OIDC config
    When a non-existent OIDC config is deleted
    Then the deletion should fail with a not found error
