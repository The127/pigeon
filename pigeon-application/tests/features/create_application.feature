Feature: Create Application

  Scenario: Rejecting an application with an empty UID
    Given a request to create an application named "my-app" with uid ""
    When the create application command is executed
    Then the command should fail with a validation error

  Scenario: Rejecting an application with a whitespace-only UID
    Given a request to create an application named "my-app" with uid "   "
    When the create application command is executed
    Then the command should fail with a validation error

  Scenario: Successfully creating an application
    Given a request to create an application named "my-app" with uid "app_123"
    When the create application command is executed
    Then the application should be created with name "my-app"
    And the application should have uid "app_123"
    And the application should have a generated id
    And the application store should contain the application

  Scenario: Rejecting an application with an empty name
    Given a request to create an application named "" with uid "app_123"
    When the create application command is executed
    Then the command should fail with a validation error

  Scenario: Rejecting an application with a whitespace-only name
    Given a request to create an application named "   " with uid "app_456"
    When the create application command is executed
    Then the command should fail with a validation error
