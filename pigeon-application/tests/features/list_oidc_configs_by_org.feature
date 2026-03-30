Feature: List OIDC Configs by Organization

  List OIDC configurations for an organization with pagination.

  Scenario: Listing OIDC configs when none exist
    Given an organization with no OIDC configs
    When the list OIDC configs query is executed
    Then the result should be an empty OIDC config list

  Scenario: Listing OIDC configs with results
    Given an organization with 2 OIDC configs
    When the list OIDC configs query is executed
    Then the result should contain 2 OIDC configs

  Scenario: Listing OIDC configs respects pagination
    Given an organization with 2 OIDC configs
    When the list OIDC configs query is executed with offset 0 and limit 1
    Then the result should contain 1 OIDC config with total 2
