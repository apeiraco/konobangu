use std::{fmt::Debug, ops::Deref};

use fetch::{HttpClient, HttpClientTrait};
use maplit::hashmap;
use scraper::{Html, Selector};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, TryIntoModel,
};
use url::Url;
use util::OptDynErr;

use super::{MikanConfig, MikanCredentialForm, constants::MIKAN_ACCOUNT_MANAGE_PAGE_PATH};
use crate::{
    app::AppContextTrait,
    crypto::UserPassCredential,
    errors::{RecorderError, RecorderResult},
    extract::mikan::constants::{MIKAN_LOGIN_PAGE_PATH, MIKAN_LOGIN_PAGE_SEARCH},
    models::credential_3rd::{self, Credential3rdType},
};

#[derive(Debug)]
pub struct MikanClient {
    http_client: HttpClient,
    base_url: Url,
    origin_url: Url,
    userpass_credential: Option<UserPassCredential>,
}

impl MikanClient {
    pub async fn from_config(config: MikanConfig) -> Result<Self, RecorderError> {
        let http_client = HttpClient::from_config(config.http_client)?;
        let base_url = config.base_url;
        let origin_url = Url::parse(&base_url.origin().unicode_serialization())?;
        Ok(Self {
            http_client,
            base_url,
            origin_url,
            userpass_credential: None,
        })
    }

    pub async fn has_login(&self) -> RecorderResult<bool> {
        let account_manage_page_url = self.base_url.join(MIKAN_ACCOUNT_MANAGE_PAGE_PATH)?;
        let res = self.http_client.get(account_manage_page_url).send().await?;
        let status = res.status();
        if status.is_success() {
            Ok(true)
        } else if status.is_redirection()
            && res.headers().get("location").is_some_and(|location| {
                location
                    .to_str()
                    .is_ok_and(|location_str| location_str.contains(MIKAN_LOGIN_PAGE_PATH))
            })
        {
            Ok(false)
        } else {
            Err(RecorderError::Credential3rdError {
                message: format!("mikan account check has login failed, status = {status}"),
                source: None.into(),
            })
        }
    }

    pub async fn login(&self) -> RecorderResult<()> {
        let userpass_credential =
            self.userpass_credential
                .as_ref()
                .ok_or_else(|| RecorderError::Credential3rdError {
                    message: "mikan login failed, credential required".to_string(),
                    source: None.into(),
                })?;

        let login_page_url = {
            let mut u = self.base_url.join(MIKAN_LOGIN_PAGE_PATH)?;
            u.set_query(Some(MIKAN_LOGIN_PAGE_SEARCH));
            u
        };

        let antiforgery_token = {
            // access login page to get antiforgery cookie
            let login_page_html = self
                .http_client
                .get(login_page_url.clone())
                .send()
                .await
                .map_err(|error| RecorderError::Credential3rdError {
                    message: "failed to get mikan login page".to_string(),
                    source: OptDynErr::some_boxed(error),
                })?
                .text()
                .await?;

            let login_page_html = Html::parse_document(&login_page_html);

            let antiforgery_selector =
                Selector::parse("input[name='__RequestVerificationToken']").unwrap();

            login_page_html
                .select(&antiforgery_selector)
                .next()
                .and_then(|element| element.value().attr("value").map(|value| value.to_string()))
                .ok_or_else(|| RecorderError::Credential3rdError {
                    message: "mikan login failed, failed to get antiforgery token".to_string(),
                    source: None.into(),
                })
        }?;

        let login_post_form = hashmap! {
            "__RequestVerificationToken".to_string() => antiforgery_token,
            "UserName".to_string() => userpass_credential.username.clone(),
            "Password".to_string() => userpass_credential.password.clone(),
            "RememberMe".to_string() => "true".to_string(),
        };
        let login_post_res = self
            .http_client
            .post(login_page_url.clone())
            .form(&login_post_form)
            .send()
            .await
            .map_err(|err| RecorderError::Credential3rdError {
                message: "mikan login failed".to_string(),
                source: OptDynErr::some_boxed(err),
            })?;

        if login_post_res.status().is_redirection()
            && login_post_res.headers().contains_key("location")
        {
            Ok(())
        } else {
            Err(RecorderError::Credential3rdError {
                message: "mikan login failed, no redirecting".to_string(),
                source: None.into(),
            })
        }
    }

    pub async fn submit_credential_form(
        &self,
        ctx: &dyn AppContextTrait,
        subscriber_id: i32,
        credential_form: MikanCredentialForm,
    ) -> RecorderResult<credential_3rd::Model> {
        let db = ctx.db();
        let am = credential_3rd::ActiveModel {
            username: Set(Some(credential_form.username)),
            password: Set(Some(credential_form.password)),
            user_agent: Set(Some(credential_form.user_agent)),
            credential_type: Set(Credential3rdType::Mikan),
            subscriber_id: Set(subscriber_id),
            ..Default::default()
        }
        .try_encrypt(ctx)
        .await?;

        let credential: credential_3rd::Model = am.save(db).await?.try_into_model()?;
        Ok(credential)
    }

    pub async fn sync_credential_cookies(
        &self,
        ctx: &dyn AppContextTrait,
        credential_id: i32,
        subscriber_id: i32,
    ) -> RecorderResult<()> {
        let cookies = self.http_client.save_cookie_store_to_json()?;
        if let Some(cookies) = cookies {
            let am = credential_3rd::ActiveModel {
                cookies: Set(Some(cookies)),
                ..Default::default()
            }
            .try_encrypt(ctx)
            .await?;

            credential_3rd::Entity::update_many()
                .set(am)
                .filter(credential_3rd::Column::Id.eq(credential_id))
                .filter(credential_3rd::Column::SubscriberId.eq(subscriber_id))
                .exec(ctx.db())
                .await?;
        }
        Ok(())
    }

    pub async fn fork_with_userpass_credential(
        &self,
        userpass_credential: UserPassCredential,
    ) -> RecorderResult<Self> {
        let mut fork = self
            .http_client
            .fork()
            .attach_cookies(userpass_credential.cookies.as_deref())?;

        if let Some(user_agent) = userpass_credential.user_agent.as_ref() {
            fork = fork.attach_user_agent(user_agent);
        }

        let userpass_credential_opt = Some(userpass_credential);

        Ok(Self {
            http_client: HttpClient::from_fork(fork)?,
            base_url: self.base_url.clone(),
            origin_url: self.origin_url.clone(),
            userpass_credential: userpass_credential_opt,
        })
    }

    pub async fn fork_with_credential_id(
        &self,
        ctx: &dyn AppContextTrait,
        credential_id: i32,
        subscriber_id: i32,
    ) -> RecorderResult<Self> {
        let credential =
            credential_3rd::Model::find_by_id_and_subscriber_id(ctx, credential_id, subscriber_id)
                .await?;
        if let Some(credential) = credential {
            if credential.credential_type != Credential3rdType::Mikan {
                return Err(RecorderError::Credential3rdError {
                    message: "credential is not a mikan credential".to_string(),
                    source: None.into(),
                });
            }

            let userpass_credential: UserPassCredential =
                credential.try_into_userpass_credential(ctx)?;

            self.fork_with_userpass_credential(userpass_credential)
                .await
        } else {
            Err(RecorderError::from_entity_not_found_detail::<
                credential_3rd::Entity,
                _,
            >(format!(
                "credential id {credential_id} not found"
            )))
        }
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn client(&self) -> &HttpClient {
        &self.http_client
    }
}

impl Deref for MikanClient {
    type Target = fetch::reqwest_middleware::ClientWithMiddleware;

    fn deref(&self) -> &Self::Target {
        &self.http_client
    }
}

impl HttpClientTrait for MikanClient {}

#[cfg(test)]
mod tests {
    #![allow(unused_variables)]
    use std::{assert_matches, sync::Arc};

    use rstest::{fixture, rstest};
    use tracing::Level;

    use super::*;
    use crate::test_utils::{
        app::TestingAppContext,
        crypto::build_testing_crypto_service,
        database::build_testing_database_service,
        mikan::{MikanMockServer, build_testing_mikan_client, build_testing_mikan_credential_form},
        tracing::try_init_testing_tracing,
    };

    async fn create_testing_context(
        mikan_base_url: Url,
    ) -> RecorderResult<Arc<dyn AppContextTrait>> {
        let mikan_client = build_testing_mikan_client(mikan_base_url.clone()).await?;
        let db_service = build_testing_database_service(Default::default()).await?;
        let crypto_service = build_testing_crypto_service().await?;
        let ctx = TestingAppContext::builder()
            .db(db_service)
            .crypto(crypto_service)
            .mikan(mikan_client)
            .build();

        Ok(Arc::new(ctx))
    }

    #[fixture]
    fn before_each() {
        try_init_testing_tracing(Level::DEBUG);
    }

    #[rstest]
    #[tokio::test]
    async fn test_mikan_client_submit_credential_form(before_each: ()) -> RecorderResult<()> {
        let mut mikan_server = MikanMockServer::new().await?;

        let app_ctx = create_testing_context(mikan_server.base_url().clone()).await?;

        let _login_mock = mikan_server.mock_get_login_page();

        let mikan_client = app_ctx.mikan();
        let crypto_service = app_ctx.crypto();

        let credential_form = build_testing_mikan_credential_form();

        let subscriber_id = 1;

        let credential_model = mikan_client
            .submit_credential_form(app_ctx.as_ref(), subscriber_id, credential_form.clone())
            .await?;

        let expected_username = &credential_form.username;
        let expected_password = &credential_form.password;

        let found_username = crypto_service
            .decrypt_string(credential_model.username.as_deref().unwrap_or_default())?;
        let found_password = crypto_service
            .decrypt_string(credential_model.password.as_deref().unwrap_or_default())?;

        assert_eq!(&found_username, expected_username);
        assert_eq!(&found_password, expected_password);

        let has_login = mikan_client.has_login().await?;

        assert!(!has_login);

        assert_matches!(
            mikan_client.login().await,
            Err(RecorderError::Credential3rdError { .. })
        );

        let mikan_client = mikan_client
            .fork_with_credential_id(app_ctx.as_ref(), credential_model.id, subscriber_id)
            .await?;

        mikan_client.login().await?;

        let has_login = mikan_client.has_login().await?;

        assert!(has_login);

        Ok(())
    }
}
