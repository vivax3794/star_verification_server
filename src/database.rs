use crate::stars::StarData;
use worker::{console_log, RouteContext};

pub struct DbReference(pub String);

const STAR_KV_BINDING: &str = "stars";

pub struct CFWorkersKV {
    kv: worker::kv::KvStore,
}

impl CFWorkersKV {
    pub fn new(ctx: &RouteContext<()>) -> Self {
        Self {
            kv: ctx.kv(STAR_KV_BINDING).unwrap(),
        }
    }

    pub async fn store_stars(&mut self, stars: StarData) -> DbReference {
        let id_to_use = uuid::Uuid::new_v4();
        self.kv
            .put(&id_to_use.to_string(), stars)
            .unwrap()
            .expiration_ttl(60 * 60)
            .execute()
            .await
            .unwrap();
        console_log!("Saved using: {id_to_use}");

        DbReference(id_to_use.to_string())
    }

    pub async fn get_stars(&mut self, reference: DbReference) -> Option<StarData> {
        self.kv.get(&reference.0).json().await.unwrap()
    }
}
