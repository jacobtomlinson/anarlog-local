const UPSTREAM_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Clone)]
pub struct AppState {
    pub(crate) config: crate::config::SharedNotesConfig,
    pub(crate) client: reqwest::Client,
    pub(crate) storage: anlg_supabase_storage::SupabaseStorage,
}

impl AppState {
    pub fn new(config: crate::config::SharedNotesConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(UPSTREAM_REQUEST_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("shared notes HTTP client must build");
        let storage = anlg_supabase_storage::SupabaseStorage::new(
            client.clone(),
            &config.supabase_url,
            &config.supabase_service_role_key,
        );

        Self {
            config,
            client,
            storage,
        }
    }
}
