use std::sync::Arc;

use async_trait::async_trait;
use pigeon_domain::application::{Application, ApplicationId};
use pigeon_domain::organization::OrganizationId;
use pigeon_domain::version::Version;

use crate::error::ApplicationError;
use crate::mediator::command::Command;
use crate::mediator::handler::CommandHandler;
use crate::ports::unit_of_work::UnitOfWorkFactory;

#[derive(Debug)]
pub struct UpdateApplication {
    pub org_id: OrganizationId,
    pub id: ApplicationId,
    pub name: String,
    pub version: Version,
}

impl Command for UpdateApplication {
    type Output = Application;

    fn command_name(&self) -> &'static str {
        "UpdateApplication"
    }
}

pub struct UpdateApplicationHandler {
    uow_factory: Arc<dyn UnitOfWorkFactory>,
}

impl UpdateApplicationHandler {
    pub fn new(uow_factory: Arc<dyn UnitOfWorkFactory>) -> Self {
        Self { uow_factory }
    }
}

#[async_trait]
impl CommandHandler<UpdateApplication> for UpdateApplicationHandler {
    async fn handle(&self, command: UpdateApplication) -> Result<Application, ApplicationError> {
        let mut uow = self.uow_factory.begin().await?;

        let mut app = uow
            .application_store()
            .find_by_id(&command.id, &command.org_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        if app.version() != command.version {
            return Err(ApplicationError::Conflict);
        }

        app.rename(command.name)
            .map_err(|e| ApplicationError::Validation(e.to_string()))?;

        uow.application_store().save(&app).await?;
        uow.commit().await?;

        Ok(app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fakes::{FakeUnitOfWorkFactory, OperationLog};
    use pigeon_domain::application::Application;
    use pigeon_domain::organization::OrganizationId;

    fn setup_with_app() -> (OperationLog, Arc<FakeUnitOfWorkFactory>, Application, OrganizationId) {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log.clone()));
        let org_id = OrganizationId::new();
        let app = Application::new(org_id.clone(), "original-name".into(), "app_123".into()).unwrap();
        (log, factory, app, org_id)
    }

    #[tokio::test]
    async fn updates_application_successfully() {
        let (log, factory, app, org_id) = setup_with_app();
        let id = app.id().clone();
        let version = app.version();

        // Pre-seed the store
        {
            let mut uow = factory.begin().await.unwrap();
            uow.application_store().insert(&app).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateApplicationHandler::new(factory);
        let result = handler
            .handle(UpdateApplication {
                org_id,
                id,
                name: "new-name".into(),
                version,
            })
            .await;

        let updated = result.unwrap();
        assert_eq!(updated.name(), "new-name");
        assert_eq!(
            log.entries(),
            vec![
                "uow_factory:begin",
                "application_store:find_by_id",
                "application_store:save",
                "uow:commit",
            ]
        );
    }

    #[tokio::test]
    async fn returns_not_found_when_application_does_not_exist() {
        let log = OperationLog::new();
        let factory = Arc::new(FakeUnitOfWorkFactory::new(log));

        let handler = UpdateApplicationHandler::new(factory);
        let result = handler
            .handle(UpdateApplication {
                org_id: OrganizationId::new(),
                id: ApplicationId::new(),
                name: "new-name".into(),
                version: Version::new(0),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }

    #[tokio::test]
    async fn returns_not_found_for_wrong_org() {
        let (log, factory, app, _org_id) = setup_with_app();
        let id = app.id().clone();
        let version = app.version();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.application_store().insert(&app).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateApplicationHandler::new(factory);
        let result = handler
            .handle(UpdateApplication {
                org_id: OrganizationId::new(), // different org
                id,
                name: "new-name".into(),
                version,
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::NotFound)));
    }

    #[tokio::test]
    async fn rejects_empty_name() {
        let (log, factory, app, org_id) = setup_with_app();
        let id = app.id().clone();
        let version = app.version();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.application_store().insert(&app).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateApplicationHandler::new(factory);
        let result = handler
            .handle(UpdateApplication {
                org_id,
                id,
                name: "".into(),
                version,
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::Validation(_))));
    }

    #[tokio::test]
    async fn rejects_version_conflict() {
        let (log, factory, app, org_id) = setup_with_app();
        let id = app.id().clone();

        {
            let mut uow = factory.begin().await.unwrap();
            uow.application_store().insert(&app).await.unwrap();
            uow.commit().await.unwrap();
        }
        log.entries.lock().unwrap().clear();

        let handler = UpdateApplicationHandler::new(factory);
        let result = handler
            .handle(UpdateApplication {
                org_id,
                id,
                name: "new-name".into(),
                version: Version::new(999),
            })
            .await;

        assert!(matches!(result, Err(ApplicationError::Conflict)));
    }
}
