Feature: Endpoint CRUD

  Scenario: Rejecting an endpoint with an invalid URL scheme
    Given a request to create an endpoint with url "ftp://example.com/webhook" for an application
    When the create endpoint command is executed
    Then the create endpoint command should fail with a validation error

  Scenario: Rejecting an endpoint with non-existent event types
    Given a request to create an endpoint with a non-existent event type
    When the create endpoint command is executed
    Then the create endpoint command should fail with a validation error

  Scenario: Successfully creating an endpoint
    Given a request to create an endpoint with url "https://example.com/webhook" for an application
    When the create endpoint command is executed
    Then the endpoint should be created with url "https://example.com/webhook"
    And the endpoint should have a generated id
    And the endpoint store should contain the endpoint

  Scenario: Rejecting an endpoint with an empty url
    Given a request to create an endpoint with url "" for an application
    When the create endpoint command is executed
    Then the create endpoint command should fail with a validation error

  Scenario: Rejecting an endpoint with a whitespace-only url
    Given a request to create an endpoint with url "   " for an application
    When the create endpoint command is executed
    Then the create endpoint command should fail with a validation error

  Scenario: Creating an endpoint without a signing secret
    Given a request to create an endpoint with url "https://example.com/hook" and signing secret ""
    When the create endpoint command is executed
    Then the endpoint should be created with url "https://example.com/hook"

  Scenario: Successfully updating an endpoint
    Given an existing endpoint with url "https://example.com/webhook"
    When the update endpoint command is executed with url "https://new.example.com/webhook"
    Then the endpoint should be updated with url "https://new.example.com/webhook"
    And the endpoint store should have saved the endpoint

  Scenario: Rejecting update with non-existent event types
    Given an existing endpoint with url "https://example.com/webhook"
    When the update endpoint command is executed with a non-existent event type
    Then the update endpoint command should fail with a validation error

  Scenario: Rejecting update with an empty url
    Given an existing endpoint with url "https://example.com/webhook"
    When the update endpoint command is executed with url ""
    Then the update endpoint command should fail with a validation error

  Scenario: Rejecting update with a whitespace-only url
    Given an existing endpoint with url "https://example.com/webhook"
    When the update endpoint command is executed with url "   "
    Then the update endpoint command should fail with a validation error

  Scenario: Rejecting update for a non-existent endpoint
    When the update endpoint command is executed for a non-existent endpoint
    Then the update endpoint command should fail with a not found error

  Scenario: Update endpoint with version conflict
    Given an existing endpoint with url "https://example.com/webhook"
    When the update endpoint command is executed with a stale version
    Then the update endpoint command should fail with a conflict error

  Scenario: Successfully deleting an endpoint
    Given an endpoint with url "https://example.com/webhook" exists
    When the delete endpoint command is executed
    Then the endpoint should be deleted successfully
    And the endpoint store should have deleted the endpoint

  Scenario: Deleting a non-existent endpoint
    When the delete endpoint command is executed for a non-existent endpoint
    Then the delete endpoint command should fail with a not found error
