Feature: Create Organization

  Scenario: Successfully creating an organization
    Given a request to create an organization named "my-org" with slug "my-org"
    When the create organization command is executed
    Then the organization should be created with name "my-org"
    And the organization should have slug "my-org"
    And the organization should have a generated id
    And the organization store should contain the organization

  Scenario: Rejecting an organization with an empty name
    Given a request to create an organization named "" with slug "my-org"
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Rejecting an organization with a whitespace-only name
    Given a request to create an organization named "   " with slug "my-org"
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Rejecting an organization with an empty slug
    Given a request to create an organization named "my-org" with slug ""
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Rejecting an organization with a whitespace-only slug
    Given a request to create an organization named "my-org" with slug "   "
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Rejecting an organization with an invalid slug
    Given a request to create an organization named "my-org" with slug "My Org"
    When the create organization command is executed
    Then the create organization command should fail with a validation error

  Scenario: Accepting a slug with hyphens and digits
    Given a request to create an organization named "my-org" with slug "my-org-123"
    When the create organization command is executed
    Then the organization should be created with name "my-org"
    And the organization should have slug "my-org-123"
