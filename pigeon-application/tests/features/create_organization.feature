Feature: Create Organization

  Organizations must always have an OIDC configuration.

  Scenario: Rejecting a duplicate slug
    Given an organization with slug "taken-slug" already exists
    When the create organization command is executed with slug "taken-slug"
    Then the create organization command should fail with a validation error

  Scenario: Successfully creating an organization with OIDC config
    Given a request to create an organization named "my-org" with slug "my-org"
    And OIDC config with issuer "https://auth.example.com" and audience "pigeon-api"
    When the create organization command is executed
    Then the organization should be created with name "my-org"
    And the organization should have slug "my-org"
    And the organization should have a generated id
    And the organization store should contain the organization
    And the OIDC config store should contain a config for the organization

  Scenario: Rejecting an organization with an empty name
    Given a request to create an organization named "" with slug "my-org"
    And OIDC config with issuer "https://auth.example.com" and audience "pigeon-api"
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Rejecting an organization with a whitespace-only name
    Given a request to create an organization named "   " with slug "my-org"
    And OIDC config with issuer "https://auth.example.com" and audience "pigeon-api"
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Rejecting an organization with an empty slug
    Given a request to create an organization named "my-org" with slug ""
    And OIDC config with issuer "https://auth.example.com" and audience "pigeon-api"
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Rejecting an organization with a whitespace-only slug
    Given a request to create an organization named "my-org" with slug "   "
    And OIDC config with issuer "https://auth.example.com" and audience "pigeon-api"
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Rejecting an organization with an invalid slug
    Given a request to create an organization named "my-org" with slug "My Org"
    And OIDC config with issuer "https://auth.example.com" and audience "pigeon-api"
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Accepting a slug with hyphens and digits
    Given a request to create an organization named "my-org" with slug "my-org-123"
    And OIDC config with issuer "https://auth.example.com" and audience "pigeon-api"
    When the create organization command is executed
    Then the organization should be created with name "my-org"
    And the organization should have slug "my-org-123"

  Scenario: Rejecting an organization with an empty OIDC issuer
    Given a request to create an organization named "my-org" with slug "my-org"
    And OIDC config with issuer "" and audience "pigeon-api"
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Rejecting an organization with an empty OIDC audience
    Given a request to create an organization named "my-org" with slug "my-org"
    And OIDC config with issuer "https://auth.example.com" and audience ""
    When the create organization command is executed
    Then the create organization command should fail with a validation error
